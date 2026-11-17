import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertionToJSON,
  base64urlToBuffer,
  bufferToBase64url,
  passkeyErrorMessage,
  registrationToJSON,
  toCreationOptions,
  toRequestOptions,
} from "../app/utils/webauthn.ts";

const bytes = (...values: number[]) => new Uint8Array(values).buffer;

test("base64url round-trips without padding", () => {
  const buffer = bytes(0, 1, 2, 250, 251, 252, 253, 254, 255);
  const encoded = bufferToBase64url(buffer);

  assert.equal(encoded, "AAEC-vv8_f7_");
  assert.deepEqual(new Uint8Array(base64urlToBuffer(encoded)), new Uint8Array(buffer));
  assert.deepEqual(new Uint8Array(base64urlToBuffer("AAEC-vv8_f7_")), new Uint8Array(buffer));
  assert.deepEqual(new Uint8Array(base64urlToBuffer("AQ==")), new Uint8Array([1]));
});

test("toCreationOptions decodes the binary fields from webauthn-rs JSON", () => {
  const options = toCreationOptions({
    publicKey: {
      rp: { id: "example.com", name: "Kakeibo" },
      user: { id: "AQID", name: "kaito", displayName: "kaito" },
      challenge: "BAUG",
      pubKeyCredParams: [{ type: "public-key", alg: -7 }],
      excludeCredentials: [{ type: "public-key", id: "Bwg" }],
      authenticatorSelection: { residentKey: "required", userVerification: "required" },
    },
  });

  assert.deepEqual(new Uint8Array(options.user.id as ArrayBuffer), new Uint8Array([1, 2, 3]));
  assert.deepEqual(new Uint8Array(options.challenge as ArrayBuffer), new Uint8Array([4, 5, 6]));
  assert.deepEqual(
    new Uint8Array(options.excludeCredentials![0]!.id as ArrayBuffer),
    new Uint8Array([7, 8]),
  );
  assert.equal(options.rp.name, "Kakeibo");
  assert.equal(options.authenticatorSelection?.residentKey, "required");
});

test("toRequestOptions decodes the challenge and allowed credentials", () => {
  const options = toRequestOptions({
    publicKey: {
      challenge: "BAUG",
      rpId: "example.com",
      allowCredentials: [],
      userVerification: "required",
    },
  });

  assert.deepEqual(new Uint8Array(options.challenge as ArrayBuffer), new Uint8Array([4, 5, 6]));
  assert.deepEqual(options.allowCredentials, []);
  assert.equal(options.rpId, "example.com");
});

test("registrationToJSON encodes the attestation for the server", () => {
  const credential = {
    id: "Bwg",
    rawId: bytes(7, 8),
    type: "public-key",
    response: {
      attestationObject: bytes(1),
      clientDataJSON: bytes(2),
      getTransports: () => ["internal", "hybrid"],
    },
    getClientExtensionResults: () => ({ credProps: { rk: true } }),
  };

  assert.deepEqual(registrationToJSON(credential as unknown as PublicKeyCredential), {
    id: "Bwg",
    rawId: "Bwg",
    type: "public-key",
    response: { attestationObject: "AQ", clientDataJSON: "Ag", transports: ["internal", "hybrid"] },
    clientExtensionResults: { credProps: { rk: true } },
  });
});

test("assertionToJSON encodes the signature and user handle", () => {
  const credential = {
    id: "Bwg",
    rawId: bytes(7, 8),
    type: "public-key",
    response: {
      authenticatorData: bytes(1),
      clientDataJSON: bytes(2),
      signature: bytes(3),
      userHandle: bytes(4),
    },
    getClientExtensionResults: () => ({}),
  };

  assert.deepEqual(assertionToJSON(credential as unknown as PublicKeyCredential), {
    id: "Bwg",
    rawId: "Bwg",
    type: "public-key",
    response: {
      authenticatorData: "AQ",
      clientDataJSON: "Ag",
      signature: "Aw",
      userHandle: "BA",
    },
    clientExtensionResults: {},
  });
});

test("passkeyErrorMessage tells cancellations apart from failures", () => {
  assert.equal(
    passkeyErrorMessage(new DOMException("cancelled", "NotAllowedError")),
    "パスキーの操作がキャンセルされたか、時間切れになりました。",
  );
  assert.match(
    passkeyErrorMessage(new DOMException("dup", "InvalidStateError")),
    /すでに登録されています/,
  );
  assert.match(passkeyErrorMessage({ statusCode: 401 }), /このパスキーではログインできません/);
  assert.match(passkeyErrorMessage(new Error("other")), /パスキーを使えませんでした/);
});
