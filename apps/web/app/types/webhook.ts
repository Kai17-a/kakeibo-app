export interface WebhookUrl {
  id: string;
  created_at: string;
  updated_at: string;
  url: string;
  description: string | null;
  is_active: boolean;
  events: string[];
}

export interface WebhookUrlInput {
  url: string;
  description: string | null;
  is_active: boolean;
  events: string[];
}
