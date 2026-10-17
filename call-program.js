const {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require("@solana/web3.js");

const fs = require("fs");
const os = require("os");
const path = require("path");

async function main() {
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  const programId = new PublicKey(
    "HPhdXbDzyoQQVjA94o6JWpd1zP5bJD5PpgwtWeeRwKx3"
  );

  const secretKey = Uint8Array.from(
      JSON.parse(
          fs.readFileSync(
              path.join(os.homedir(), ".config", "solana", "id.json"),
              "utf8"
          )
      )
  );
  const payer = Keypair.fromSecretKey(secretKey);

  const instruction = new TransactionInstruction({
    programId,
    keys: [
      {
        pubkey: programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    data: Buffer.alloc(10),
  });

  const transaction = new Transaction().add(instruction);

  const signature = await sendAndConfirmTransaction(connection, transaction, [
    payer,
  ]);

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