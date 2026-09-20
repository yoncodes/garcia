INSERT INTO player_equip_deputy_words (uid, user_equip_id, position, word_id)
SELECT planned.uid,
       planned.user_equip_id,
       COALESCE((
           SELECT MAX(active.position) + 1
           FROM player_equip_deputy_words active
           WHERE active.uid = planned.uid
             AND active.user_equip_id = planned.user_equip_id
       ), 0) + planned.position,
       planned.word_id
FROM player_equip_planned_words planned;

DROP TABLE player_equip_planned_words;
