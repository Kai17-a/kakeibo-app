CREATE TABLE webhook_url_events (
  webhook_url_id TEXT NOT NULL REFERENCES webhook_urls(id) ON DELETE CASCADE,
  event TEXT NOT NULL,
  PRIMARY KEY (webhook_url_id, event)
);

INSERT INTO webhook_url_events (webhook_url_id, event)
SELECT id, 'expense.created' FROM webhook_urls
UNION ALL SELECT id, 'income.created' FROM webhook_urls
UNION ALL SELECT id, 'budget.exceeded' FROM webhook_urls;
