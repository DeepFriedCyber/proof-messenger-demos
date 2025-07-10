import React, { useState } from "react";
import init, { generate_keypair_js, make_proof_js } from "proof-messenger-protocol-wasm";

export default function App() {
  const [pubkey, setPubkey] = useState<string | null>(null);

  async function handleGenKey() {
    await init();
    const key = generate_keypair_js();
    setPubkey(Buffer.from(key).toString("hex"));
  }

  return (
    <div>
      <h1>ProofPipe Web Demo</h1>
      <button onClick={handleGenKey}>Generate Keypair</button>
      {pubkey && <div>Public Key: {pubkey}</div>}
      {/* Add onboarding, messaging, proof details here */}
    </div>
  );
}