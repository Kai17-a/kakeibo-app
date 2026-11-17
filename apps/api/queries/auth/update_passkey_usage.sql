UPDATE passkeys SET credential = ?3, last_used_at = ?4
WHERE
  id = ?1 AND user_id = ?2;
