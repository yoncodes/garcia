use std::collections::VecDeque;

use super::error::{KcpInputError, KcpSendError};

const NO_DELAY_RETRANSMIT_TIMEOUT: u32 = 30;
const DEFAULT_RETRANSMIT_TIMEOUT: u32 = 200;
const MAX_RETRANSMIT_TIMEOUT: u32 = 60_000;
const COMMAND_PUSH: u8 = 81;
const COMMAND_ACK: u8 = 82;
const COMMAND_WINDOW_PROBE: u8 = 83;
const COMMAND_WINDOW_SIZE: u8 = 84;
const PROBE_WINDOW_SIZE: u32 = 1;
const REPORT_WINDOW_SIZE: u32 = 2;
const MAX_TRANSMISSION_UNIT: usize = 1_400;
pub const HEADER_SIZE: usize = 24;
const DEFAULT_WINDOW_SIZE: u32 = 256;
const INITIAL_PROBE_INTERVAL: u32 = 7_000;
const MAX_PROBE_INTERVAL: u32 = 120_000;

#[derive(Debug, Default)]
struct Segment {
    conv: u32,
    cmd: u8,
    frg: u8,
    wnd: u16,
    ts: u32,
    sn: u32,
    una: u32,
    rto: u32,
    xmit: u32,
    resend_ts: u32,
    fast_ack: u32,
    acked: bool,
    data: Vec<u8>,
}

impl Segment {
    fn encode(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.conv.to_le_bytes());
        output.push(self.cmd);
        output.push(self.frg);
        output.extend_from_slice(&self.wnd.to_le_bytes());
        output.extend_from_slice(&self.ts.to_le_bytes());
        output.extend_from_slice(&self.sn.to_le_bytes());
        output.extend_from_slice(&self.una.to_le_bytes());
        output.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
    }
}

pub struct Kcp {
    conv: u32,
    state: u32,
    snd_una: u32,
    snd_nxt: u32,
    rcv_nxt: u32,
    ssthresh: u32,
    rx_rttval: i32,
    rx_srtt: i32,
    rx_rto: u32,
    rx_min_rto: u32,
    snd_wnd: u32,
    rcv_wnd: u32,
    rmt_wnd: u32,
    cwnd: u32,
    probe: u32,
    interval: u32,
    ts_flush: u32,
    updated: bool,
    ts_probe: u32,
    probe_wait: u32,
    dead_link: u32,
    incr: u32,
    fast_resend: u32,
    no_cwnd: bool,
    snd_queue: VecDeque<Segment>,
    rcv_queue: VecDeque<Segment>,
    snd_buf: VecDeque<Segment>,
    rcv_buf: VecDeque<Segment>,
    ack_list: Vec<(u32, u32)>,
    output: Vec<Vec<u8>>,
}

impl Kcp {
    pub fn new(conv: u32) -> Self {
        Self {
            conv,
            state: 0,
            snd_una: 0,
            snd_nxt: 0,
            rcv_nxt: 0,
            ssthresh: 2,
            rx_rttval: 0,
            rx_srtt: 0,
            rx_rto: DEFAULT_RETRANSMIT_TIMEOUT,
            rx_min_rto: NO_DELAY_RETRANSMIT_TIMEOUT,
            snd_wnd: DEFAULT_WINDOW_SIZE,
            rcv_wnd: DEFAULT_WINDOW_SIZE,
            rmt_wnd: 32,
            cwnd: 0,
            probe: 0,
            interval: 10,
            ts_flush: 100,
            updated: false,
            ts_probe: 0,
            probe_wait: 0,
            dead_link: 20,
            incr: 0,
            fast_resend: 2,
            no_cwnd: true,
            snd_queue: VecDeque::with_capacity(16),
            rcv_queue: VecDeque::with_capacity(16),
            snd_buf: VecDeque::with_capacity(16),
            rcv_buf: VecDeque::with_capacity(16),
            ack_list: Vec::with_capacity(16),
            output: Vec::new(),
        }
    }

    pub fn conversation(data: &[u8]) -> Option<u32> {
        let header = data.get(..HEADER_SIZE)?;
        if !(COMMAND_PUSH..=COMMAND_WINDOW_SIZE).contains(&header[4]) {
            return None;
        }
        let payload_len = read_u32(header, 20) as usize;
        (data.len() >= HEADER_SIZE + payload_len)
            .then(|| u32::from_le_bytes(header[..4].try_into().unwrap()))
    }

    pub fn is_dead(&self) -> bool {
        self.state == u32::MAX
    }

    pub fn send(&mut self, data: &[u8], now: u32) -> Result<(), KcpSendError> {
        if data.is_empty() {
            return Err(KcpSendError::Empty);
        }
        let mss = MAX_TRANSMISSION_UNIT - HEADER_SIZE;
        let count = data.len().div_ceil(mss);
        if count > u8::MAX as usize {
            return Err(KcpSendError::TooLarge { fragments: count });
        }
        for (index, chunk) in data.chunks(mss).enumerate() {
            self.snd_queue.push_back(Segment {
                frg: (count - index - 1) as u8,
                data: chunk.to_vec(),
                ..Segment::default()
            });
        }
        self.flush(now, false);
        Ok(())
    }

    pub fn input(&mut self, data: &[u8], now: u32) -> Result<(), KcpInputError> {
        if data.len() < HEADER_SIZE {
            return Err(KcpInputError::TooShort);
        }
        let old_una = self.snd_una;
        let mut offset = 0;
        let mut last_ack_ts = None;
        let mut una_advanced = false;

        while data.len() - offset >= HEADER_SIZE {
            let conv = read_u32(data, offset);
            if conv != self.conv {
                return Err(KcpInputError::WrongConversation);
            }
            let cmd = data[offset + 4];
            let frg = data[offset + 5];
            let wnd = read_u16(data, offset + 6);
            let ts = read_u32(data, offset + 8);
            let sn = read_u32(data, offset + 12);
            let una = read_u32(data, offset + 16);
            let length = read_u32(data, offset + 20) as usize;
            offset += HEADER_SIZE;
            if data.len() - offset < length {
                return Err(KcpInputError::TruncatedPayload);
            }
            if !(COMMAND_PUSH..=COMMAND_WINDOW_SIZE).contains(&cmd) {
                return Err(KcpInputError::UnknownCommand(cmd));
            }

            self.rmt_wnd = wnd as u32;
            una_advanced |= self.discard_acknowledged(una);
            self.update_oldest_unacknowledged();

            match cmd {
                COMMAND_ACK => {
                    self.mark_acknowledged(sn);
                    self.count_fast_acknowledgements(sn, ts);
                    last_ack_ts = Some(ts);
                }
                COMMAND_PUSH => {
                    if time_diff(sn, self.rcv_nxt.wrapping_add(self.rcv_wnd)) < 0 {
                        self.ack_list.push((sn, ts));
                        if time_diff(sn, self.rcv_nxt) >= 0 {
                            self.queue_received(Segment {
                                conv,
                                cmd,
                                frg,
                                wnd,
                                ts,
                                sn,
                                una,
                                data: data[offset..offset + length].to_vec(),
                                ..Segment::default()
                            });
                        }
                    }
                }
                COMMAND_WINDOW_PROBE => self.probe |= REPORT_WINDOW_SIZE,
                COMMAND_WINDOW_SIZE => {}
                _ => unreachable!(),
            }
            offset += length;
        }

        if let Some(ts) = last_ack_ts {
            let rtt = time_diff(now, ts);
            if rtt >= 0 {
                self.update_retransmit_timeout(rtt);
            }
        }

        if !self.no_cwnd && time_diff(self.snd_una, old_una) > 0 && self.cwnd < self.rmt_wnd {
            self.update_congestion_window();
        }
        if una_advanced {
            self.flush(now, false);
        }
        Ok(())
    }

    pub fn recv(&mut self) -> Option<Vec<u8>> {
        let size = self.peek_size()?;
        let was_full = self.rcv_queue.len() >= self.rcv_wnd as usize;
        let mut output = Vec::with_capacity(size);
        loop {
            let segment = self.rcv_queue.pop_front()?;
            output.extend_from_slice(&segment.data);
            if segment.frg == 0 {
                break;
            }
        }

        while self.rcv_queue.len() < self.rcv_wnd as usize {
            let Some(segment) = self.rcv_buf.front() else {
                break;
            };
            if segment.sn != self.rcv_nxt {
                break;
            }
            self.rcv_nxt = self.rcv_nxt.wrapping_add(1);
            self.rcv_queue.push_back(self.rcv_buf.pop_front().unwrap());
        }
        if was_full && self.rcv_queue.len() < self.rcv_wnd as usize {
            self.probe |= REPORT_WINDOW_SIZE;
        }
        Some(output)
    }

    pub fn update(&mut self, now: u32) {
        if !self.updated {
            self.updated = true;
            self.ts_flush = now;
        }
        let mut elapsed = time_diff(now, self.ts_flush);
        if !(-10_000..10_000).contains(&elapsed) {
            self.ts_flush = now;
            elapsed = 0;
        }
        if elapsed >= 0 {
            self.ts_flush = self.ts_flush.wrapping_add(self.interval);
            if time_diff(now, self.ts_flush) >= 0 {
                self.ts_flush = now.wrapping_add(self.interval);
            }
            self.flush(now, false);
        }
    }

    pub fn take_output(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.output)
    }

    fn peek_size(&self) -> Option<usize> {
        let first = self.rcv_queue.front()?;
        if first.frg == 0 {
            return Some(first.data.len());
        }
        if self.rcv_queue.len() < first.frg as usize + 1 {
            return None;
        }
        Some(
            self.rcv_queue
                .iter()
                .take_while(|segment| segment.frg != 0)
                .map(|segment| segment.data.len())
                .sum::<usize>()
                + self
                    .rcv_queue
                    .iter()
                    .find(|segment| segment.frg == 0)
                    .map_or(0, |segment| segment.data.len()),
        )
    }

    fn update_retransmit_timeout(&mut self, rtt: i32) {
        if self.rx_srtt == 0 {
            self.rx_srtt = rtt;
            self.rx_rttval = rtt >> 1;
        } else {
            let mut delta = rtt - self.rx_srtt;
            self.rx_srtt += delta >> 3;
            if delta < 0 {
                delta = -delta;
            }
            if rtt < self.rx_srtt - self.rx_rttval {
                self.rx_rttval += (delta - self.rx_rttval) >> 5;
            } else {
                self.rx_rttval += (delta - self.rx_rttval) >> 2;
            }
        }
        let rto = self.rx_srtt + self.interval.max((self.rx_rttval as u32) << 2) as i32;
        self.rx_rto = rto.clamp(self.rx_min_rto as i32, MAX_RETRANSMIT_TIMEOUT as i32) as u32;
    }

    fn update_oldest_unacknowledged(&mut self) {
        self.snd_una = self
            .snd_buf
            .front()
            .map_or(self.snd_nxt, |segment| segment.sn);
    }

    fn mark_acknowledged(&mut self, sn: u32) {
        if time_diff(sn, self.snd_una) < 0 || time_diff(sn, self.snd_nxt) >= 0 {
            return;
        }
        for segment in &mut self.snd_buf {
            if sn == segment.sn {
                segment.acked = true;
                break;
            }
            if time_diff(sn, segment.sn) < 0 {
                break;
            }
        }
    }

    fn count_fast_acknowledgements(&mut self, sn: u32, ts: u32) {
        if time_diff(sn, self.snd_una) < 0 || time_diff(sn, self.snd_nxt) >= 0 {
            return;
        }
        for segment in &mut self.snd_buf {
            if time_diff(sn, segment.sn) < 0 {
                break;
            }
            if sn != segment.sn && time_diff(segment.ts, ts) <= 0 {
                segment.fast_ack += 1;
            }
        }
    }

    fn discard_acknowledged(&mut self, una: u32) -> bool {
        let mut removed = false;
        while self
            .snd_buf
            .front()
            .is_some_and(|segment| time_diff(una, segment.sn) > 0)
        {
            self.snd_buf.pop_front();
            removed = true;
        }
        removed
    }

    fn queue_received(&mut self, segment: Segment) {
        if time_diff(segment.sn, self.rcv_nxt.wrapping_add(self.rcv_wnd)) >= 0
            || time_diff(segment.sn, self.rcv_nxt) < 0
        {
            return;
        }
        if self.rcv_buf.iter().any(|queued| queued.sn == segment.sn) {
            return;
        }
        let position = self
            .rcv_buf
            .iter()
            .position(|queued| time_diff(segment.sn, queued.sn) < 0)
            .unwrap_or(self.rcv_buf.len());
        self.rcv_buf.insert(position, segment);

        while self.rcv_queue.len() < self.rcv_wnd as usize {
            let Some(front) = self.rcv_buf.front() else {
                break;
            };
            if front.sn != self.rcv_nxt {
                break;
            }
            self.rcv_nxt = self.rcv_nxt.wrapping_add(1);
            self.rcv_queue.push_back(self.rcv_buf.pop_front().unwrap());
        }
    }

    fn flush(&mut self, now: u32, ack_only: bool) {
        let mut packet = Vec::with_capacity(MAX_TRANSMISSION_UNIT);
        let mut header = Segment {
            conv: self.conv,
            cmd: COMMAND_ACK,
            wnd: self.receive_window(),
            una: self.rcv_nxt,
            ..Segment::default()
        };

        let ack_count = self.ack_list.len();
        for (index, (sn, ts)) in self.ack_list.drain(..).enumerate() {
            if time_diff(sn, self.rcv_nxt) >= 0 || index + 1 == ack_count {
                header.sn = sn;
                header.ts = ts;
                emit_segment(&mut self.output, &mut packet, &header);
            }
        }
        if ack_only {
            emit_packet(&mut self.output, &mut packet);
            return;
        }

        if self.rmt_wnd == 0 {
            if self.probe_wait == 0 {
                self.probe_wait = INITIAL_PROBE_INTERVAL;
                self.ts_probe = now.wrapping_add(self.probe_wait);
            } else if time_diff(now, self.ts_probe) >= 0 {
                self.probe_wait = self.probe_wait.max(INITIAL_PROBE_INTERVAL);
                self.probe_wait = (self.probe_wait + self.probe_wait / 2).min(MAX_PROBE_INTERVAL);
                self.ts_probe = now.wrapping_add(self.probe_wait);
                self.probe |= PROBE_WINDOW_SIZE;
            }
        } else {
            self.ts_probe = 0;
            self.probe_wait = 0;
        }
        if self.probe & PROBE_WINDOW_SIZE != 0 {
            header.cmd = COMMAND_WINDOW_PROBE;
            emit_segment(&mut self.output, &mut packet, &header);
        }
        if self.probe & REPORT_WINDOW_SIZE != 0 {
            header.cmd = COMMAND_WINDOW_SIZE;
            emit_segment(&mut self.output, &mut packet, &header);
        }
        self.probe = 0;

        let mut window = self.snd_wnd.min(self.rmt_wnd);
        if !self.no_cwnd {
            window = window.min(self.cwnd);
        }
        let mut moved = 0;
        while time_diff(self.snd_nxt, self.snd_una.wrapping_add(window)) < 0 {
            let Some(mut segment) = self.snd_queue.pop_front() else {
                break;
            };
            segment.conv = self.conv;
            segment.cmd = COMMAND_PUSH;
            segment.sn = self.snd_nxt;
            self.snd_nxt = self.snd_nxt.wrapping_add(1);
            self.snd_buf.push_back(segment);
            moved += 1;
        }

        let resend = if self.fast_resend == 0 {
            u32::MAX
        } else {
            self.fast_resend
        };
        let mut change = false;
        let mut lost = false;
        for segment in &mut self.snd_buf {
            let mut send = false;
            if !segment.acked {
                if segment.xmit == 0 {
                    send = true;
                    segment.rto = self.rx_rto;
                    segment.resend_ts = now.wrapping_add(segment.rto);
                } else if segment.fast_ack >= resend || (segment.fast_ack > 0 && moved == 0) {
                    send = true;
                    segment.fast_ack = 0;
                    segment.rto = self.rx_rto;
                    segment.resend_ts = now.wrapping_add(segment.rto);
                    change = true;
                } else if time_diff(now, segment.resend_ts) >= 0 {
                    send = true;
                    segment.rto = segment.rto.wrapping_add(self.rx_rto / 2);
                    segment.fast_ack = 0;
                    segment.resend_ts = now.wrapping_add(segment.rto);
                    lost = true;
                }
            }
            if send {
                segment.xmit = segment.xmit.wrapping_add(1);
                segment.ts = now;
                segment.wnd = header.wnd;
                segment.una = header.una;
                emit_segment(&mut self.output, &mut packet, segment);
                if segment.xmit >= self.dead_link {
                    self.state = u32::MAX;
                }
            }
        }
        emit_packet(&mut self.output, &mut packet);

        if !self.no_cwnd {
            if change {
                self.ssthresh = ((self.snd_nxt - self.snd_una) / 2).max(2);
                self.cwnd = self.ssthresh.wrapping_add(resend);
                self.incr = self
                    .cwnd
                    .wrapping_mul((MAX_TRANSMISSION_UNIT - HEADER_SIZE) as u32);
            }
            if lost {
                self.ssthresh = (self.cwnd / 2).max(2);
                self.cwnd = 1;
                self.incr = (MAX_TRANSMISSION_UNIT - HEADER_SIZE) as u32;
            }
            if self.cwnd < 1 {
                self.cwnd = 1;
                self.incr = (MAX_TRANSMISSION_UNIT - HEADER_SIZE) as u32;
            }
        }
    }

    fn update_congestion_window(&mut self) {
        let mss = (MAX_TRANSMISSION_UNIT - HEADER_SIZE) as u32;
        if self.cwnd < self.ssthresh {
            self.cwnd += 1;
            self.incr += mss;
        } else {
            self.incr = self.incr.max(mss);
            self.incr += mss * mss / self.incr + mss / 16;
            if (self.cwnd + 1) * mss <= self.incr {
                self.cwnd = (self.incr + mss - 1) / mss.max(1);
            }
        }
        if self.cwnd > self.rmt_wnd {
            self.cwnd = self.rmt_wnd;
            self.incr = self.rmt_wnd * mss;
        }
    }

    fn receive_window(&self) -> u16 {
        self.rcv_wnd.saturating_sub(self.rcv_queue.len() as u32) as u16
    }
}

fn emit_segment(output: &mut Vec<Vec<u8>>, packet: &mut Vec<u8>, segment: &Segment) {
    let needed = HEADER_SIZE + segment.data.len();
    if packet.len() + needed > MAX_TRANSMISSION_UNIT {
        emit_packet(output, packet);
    }
    segment.encode(packet);
    packet.extend_from_slice(&segment.data);
}

fn emit_packet(output: &mut Vec<Vec<u8>>, packet: &mut Vec<u8>) {
    if !packet.is_empty() {
        output.push(std::mem::replace(
            packet,
            Vec::with_capacity(MAX_TRANSMISSION_UNIT),
        ));
    }
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap())
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn time_diff(later: u32, earlier: u32) -> i32 {
    later.wrapping_sub(earlier) as i32
}
