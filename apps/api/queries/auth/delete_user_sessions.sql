DELETE
FROM
  sessions
WHERE
  user_id = (
    SELECT
      id
    FROM
      users
    WHERE
      username = ?1
  );
