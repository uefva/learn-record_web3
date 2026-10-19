const {
    Connection,
    clusterApiUrl,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SendTransactionError,
    SystemProgram
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
        "529NN5LZgBptRJhX8zsjAMEe2BDrLyrA6H4RfFJUnj5x"
    );

    // const secretKey = Uint8Array.from([92,14,54,243,238,136,103,137,72,118,233,69,184,123,95,139,17,71,115,98,9,137,253,68,10,18,112,249,74,135,164,61,74,220,224,178,65,83,36,255,183,146,100,108,237,48,78,63,170,144,147,162,92,201,19,81,62,63,103,25,168,28,168,108]);
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

    // 种子必须和 Rust 一致：用户公钥的字节
    const [counterPda, bump] = PublicKey.findProgramAddressSync(
        [payer.publicKey.toBytes()],
        programId
    );

    console.log("Counter PDA:", counterPda.toBase58());
    console.log("Bump:", bump);

    const { SYSVAR_RENT_PUBKEY } = require("@solana/web3.js");

    const instruction = new TransactionInstruction({
        programId,
        keys: [
            { pubkey: payer.publicKey, isSigner: true, isWritable: true },
            { pubkey: counterPda, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        ],
        data: Buffer.from([0]), // 假设 Rust 约定 0 表示初始化
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