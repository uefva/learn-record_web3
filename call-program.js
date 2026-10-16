const {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require("@solana/web3.js");

const fs = require("fs");

async function main() {
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  const programId = new PublicKey(
    "Gr7n4JCtw13j1ayiN8UCixv8v4BnbsQnY2CQoVd9ffjq"
  );

  const secretKey = Uint8Array.from(
    JSON.parse(
      fs.readFileSync("/root/.config/solana/id.json", "utf8")
    )
  );
  const payer = Keypair.fromSecretKey(secretKey);

  const instruction = new TransactionInstruction({
    programId,
    keys: [],
    data: Buffer.alloc(0),
  });

  const transaction = new Transaction().add(instruction);

  const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [payer]
  );

  console.log("Transaction signature:", signature);

  const result = await connection.getTransaction(signature, {
    commitment: "confirmed",
    maxSupportedTransactionVersion: 0,
  });

  console.log("Execution logs:");
  console.log(result?.meta?.logMessages?.join("\n") ?? "No logs");
  console.log("Execution error:", result?.meta?.err ?? null);
}

main().catch((err) => {
  console.error("Call failed:", err);
  process.exitCode = 1;
});
