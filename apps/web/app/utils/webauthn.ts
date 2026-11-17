/**
 * Converts between the JSON that webauthn-rs sends and the binary values the WebAuthn browser
 * API uses. Binary fields travel as unpadded base64url strings.
 */

type Json = Record<string, unknown>;

export function bufferToBase64url(buffer: ArrayBuffer | ArrayBufferView): string {
  const bytes =
    buffer instanceof ArrayBuffer
      ? new Uint8Array(buffer)
      : new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

export function base64urlToBuffer(value: string): ArrayBuffer {
  const base64 = value.replace(/-/g, "+").replace(/_/g, "/").replace(/=+$/, "");
  const binary = atob(base64 + "=".repeat((4 - (base64.length % 4)) % 4));
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  return bytes.buffer;
}

function decodeDescriptors(value: unknown): PublicKeyCredentialDescriptor[] | undefined {
  if (!Array.isArray(value)) return undefined;
  return value.map((descriptor: Json) => ({
    ...(descriptor as unknown as PublicKeyCredentialDescriptor),
    id: base64urlToBuffer(descriptor.id as string),
  }));
}

/** `{ publicKey }` from the registration endpoint → options for `navigator.credentials.create`. */
export function toCreationOptions(json: Json): PublicKeyCredentialCreationOptions {
  const publicKey = json.publicKey as Json;
  const user = publicKey.user as Json;
  return {
    ...(publicKey as unknown as PublicKeyCredentialCreationOptions),
    challenge: base64urlToBuffer(publicKey.challenge as string),
    user: {
      ...(user as unknown as PublicKeyCredentialUserEntity),
      id: base64urlToBuffer(user.id as string),
    },
    excludeCredentials: decodeDescriptors(publicKey.excludeCredentials),
  };
}

/** `{ publicKey }` from the login challenge endpoint → options for `navigator.credentials.get`. */
export function toRequestOptions(json: Json): PublicKeyCredentialRequestOptions {
  const publicKey = json.publicKey as Json;
  return {
    ...(publicKey as unknown as PublicKeyCredentialRequestOptions),
    challenge: base64urlToBuffer(publicKey.challenge as string),
    allowCredentials: decodeDescriptors(publicKey.allowCredentials),
  };
}

export function registrationToJSON(credential: PublicKeyCredential): Json {
  const response = credential.response as AuthenticatorAttestationResponse;
  return {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: bufferToBase64url(response.attestationObject),
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      transports: response.getTransports?.() ?? [],
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
}

export function assertionToJSON(credential: PublicKeyCredential): Json {
  const response = credential.response as AuthenticatorAssertionResponse;
  return {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToBase64url(response.authenticatorData),
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      signature: bufferToBase64url(response.signature),
      userHandle: response.userHandle ? bufferToBase64url(response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
}

/** Whether this browser can use passkeys at all (it also needs HTTPS or localhost). */
export function passkeySupported(): boolean {
  return typeof window !== "undefined" && typeof window.PublicKeyCredential === "function";
}

export function passkeyErrorMessage(error: unknown): string {
  const name = error && typeof error === "object" && "name" in error ? error.name : undefined;
  if (name === "NotAllowedError" || name === "AbortError") {
    return "パスキーの操作がキャンセルされたか、時間切れになりました。";
  }
  if (name === "InvalidStateError") {
    return "このデバイスのパスキーはすでに登録されています。";
  }
  const status =
    error && typeof error === "object" && "statusCode" in error ? error.statusCode : undefined;
  if (status === 401) {
    return "このパスキーではログインできません。登録が削除されていないか確認してください。";
  }
  if (status === 400) {
    return "時間がたったため、やり直してください。";
  }
  return "パスキーを使えませんでした。もう一度お試しください。";
}
