const {
    Connection,
    clusterApiUrl,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SendTransactionError,
} = require("@solana/web3.js");

const { setGlobalDispatcher, ProxyAgent } = require("undici");
const proxy =
    process.env.HTTPS_PROXY || process.env.https_proxy ||
    process.env.HTTP_PROXY  || process.env.http_proxy;
if (proxy) {
    console.log("Using proxy:", proxy);
    setGlobalDispatcher(new ProxyAgent(proxy));
}

const fs = require("fs");
const os = require("os");
const path = require("path");

// 纯 HTTP 轮询确认，完全绕开 WebSocket
async function waitForConfirmation(
    connection,
    signature,
    lastValidBlockHeight,
    { intervalMs = 1000, timeoutMs = 60000 } = {}
) {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
        const res = await connection.getSignatureStatuses([signature], {
            searchTransactionHistory: false,
        });
        const s = res.value[0];

        if (s) {
            if (s.err) return { err: s.err };
            if (
                s.confirmationStatus === "confirmed" ||
                s.confirmationStatus === "finalized"
            ) {
                return { err: null };
            }
        } else {
            // 状态还没有，检查 blockheight 是否过期
            const currentHeight = await connection.getBlockHeight("confirmed");
            if (currentHeight > lastValidBlockHeight) {
                throw new Error(
                    `交易已过期：当前 blockheight ${currentHeight} > lastValidBlockHeight ${lastValidBlockHeight}`
                );
            }
        }

        await new Promise((r) => setTimeout(r, intervalMs));
    }
    throw new Error("确认超时");
}

async function main() {
    const connection = new Connection(clusterApiUrl("devnet"), "confirmed");

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

    const programInfo = await connection.getAccountInfo(programId);
    if (!programInfo || !programInfo.executable) {
        throw new Error(`Program ${programId.toBase58()} 不存在或不可执行`);
    }
    console.log("Program 存在且可执行 ✅");

    const balance = await connection.getBalance(payer.publicKey);
    console.log("Payer:", payer.publicKey.toBase58());
    console.log("Payer balance (lamports):", balance);
    if (balance === 0) throw new Error("devnet 余额为 0，请先 airdrop");

    const instruction = new TransactionInstruction({
        programId,
        keys: [{ pubkey: programId, isSigner: false, isWritable: false }],
        data: Buffer.alloc(10),
    });

    const { blockhash, lastValidBlockHeight } =
        await connection.getLatestBlockhash("confirmed");

    const transaction = new Transaction({
        recentBlockhash: blockhash,
        feePayer: payer.publicKey,
    }).add(instruction);
    transaction.sign(payer);

    let signature;
    try {
        signature = await connection.sendRawTransaction(transaction.serialize(), {
            skipPreflight: false,
            preflightCommitment: "confirmed",
        });
    } catch (err) {
        if (err instanceof SendTransactionError) {
            console.error("发送失败，程序日志:");
            console.error(await err.getLogs(connection));
        }
        throw err;
    }
    console.log("Transaction signature:", signature);
    console.log(
        `Explorer: https://explorer.solana.com/tx/${signature}?cluster=devnet`
    );

    // 纯 HTTP 轮询确认，不使用任何 WebSocket
    const confirmation = await waitForConfirmation(
        connection,
        signature,
        lastValidBlockHeight
    );

    if (confirmation.err) {
        console.error("交易失败:", JSON.stringify(confirmation.err, null, 2));
    } else {
        console.log("交易成功 ✅");
    }

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