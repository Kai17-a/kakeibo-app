import type { PasskeyChallenge } from "~/composables/useAuth";
import { registrationToJSON, toCreationOptions } from "~/utils/webauthn";

export interface Passkey {
  id: string;
  name: string;
  /** Unix epoch seconds. */
  created_at: number;
  last_used_at: number | null;
}

export function usePasskeys() {
  const client = $fetch.create({ retry: 0, timeout: 15000, onResponseError: notifyUnauthorized });

  async function list(): Promise<Passkey[]> {
    return client<Passkey[]>("/api/passkeys");
  }

  /** Creates a passkey on this device and registers it for the signed-in user. */
  async function register(name: string): Promise<Passkey> {
    const challenge = await client<PasskeyChallenge>("/api/passkeys/registration", {
      method: "POST",
    });
    const credential = (await navigator.credentials.create({
      publicKey: toCreationOptions(challenge.options),
    })) as PublicKeyCredential | null;
    if (!credential) throw new DOMException("No passkey was created", "NotAllowedError");
    return client<Passkey>("/api/passkeys", {
      method: "POST",
      body: {
        challenge_id: challenge.challenge_id,
        name,
        credential: registrationToJSON(credential),
      },
    });
  }

  async function remove(id: string): Promise<void> {
    await client(`/api/passkeys/${encodeURIComponent(id)}`, { method: "DELETE" });
  }

  return { list, register, remove };
}
