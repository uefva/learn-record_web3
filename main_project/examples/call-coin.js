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
        "E6huKbE3cpahat9ZCSyCTfoXbZCaywSmKdD8DhA6bxsi"
    );

    // 必须与 src/coin.rs 里的 MINT_AUTHORITY 一致
    const MINT_AUTHORITY = new PublicKey(
        "FmZvpz8Nub192NdoWDTA8MJfcPQvrqPR2dZdXhzZj3N9"
    );

    const secretKey_2 = Uint8Array.from([223,201,245,163,39,196,91,72,58,171,88,32,145,229,99,144,99,195,105,93,232,155,5,81,247,191,158,251,28,47,185,243,78,225,253,7,248,215,63,215,188,91,97,212,97,203,203,82,195,97,243,184,31,25,239,201,158,60,161,186,197,211,186,133]);
    const secretKey = Uint8Array.from(
        JSON.parse(
            fs.readFileSync(
                path.join(os.homedir(), ".config", "solana", "id.json"),
                "utf8"
            )
        )
    );
    const payer = Keypair.fromSecretKey(secretKey);


    const payer_2 = Keypair.fromSecretKey(secretKey_2);

    const programInfo = await connection.getAccountInfo(programId);
    if (!programInfo || !programInfo.executable) {
        throw new Error(`Program ${programId.toBase58()} 不存在或不可执行`);
    }
    console.log("Program 存在且可执行 ✅");

    const balance = await connection.getBalance(payer.publicKey);
    console.log("Payer:", payer.publicKey.toBase58());
    console.log("Payer balance (lamports):", balance);
    if (balance === 0) throw new Error("devnet 余额为 0，请先 airdrop");

    // 用法: node examples/call-dev.js [mint|transfer] [amount]
    const mode = (process.argv[2] || "transfer").toLowerCase();
    const amount = BigInt(process.argv[3] ?? "100");
    if (amount <= 0n) throw new Error("amount 必须大于 0");

    const amountBuf = Buffer.alloc(8);
    amountBuf.writeBigUInt64BE(amount);

    // 种子必须和 Rust 一致：b"coin" + 用户公钥的字节
    const SEED_PREFIX = Buffer.from("coin");

    const [userPda, bump] = PublicKey.findProgramAddressSync(
        [SEED_PREFIX, payer.publicKey.toBytes()],
        programId
    );

    console.log("Payer PDA:", userPda.toBase58(), "bump:", bump);

    const [userPda2, bump2] = PublicKey.findProgramAddressSync(
        [SEED_PREFIX, payer_2.publicKey.toBytes()],
        programId
    );
    console.log("Payer_2 PDA:", userPda2.toBase58(), "bump:", bump2);

    const { SYSVAR_RENT_PUBKEY } = require("@solana/web3.js");
    const tailKeys = [
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    let keys;
    let data;

    if (mode === "mint") {
        if (!payer.publicKey.equals(MINT_AUTHORITY)) {
            throw new Error(
                `mint 只能由 ${MINT_AUTHORITY.toBase58()} 发起，当前 payer 是 ${payer.publicKey.toBase58()}`
            );
        }
        keys = [
            { pubkey: payer.publicKey, isSigner: true, isWritable: true }, // 铸币者
            { pubkey: userPda, isSigner: false, isWritable: true },        // 铸币者 PDA
            ...tailKeys,
        ];
        data = Buffer.concat([Buffer.from([0x00]), amountBuf]);
        console.log(`Mint ${amount} -> ${userPda.toBase58()}`);
    } else if (mode === "transfer") {
        // 源 PDA 必须先通过 mint 初始化（Rust 侧直接按 8 字节读取）
        const sourceInfo = await connection.getAccountInfo(userPda);
        if (!sourceInfo || sourceInfo.data.length !== 8) {
            throw new Error(
                `源 PDA ${userPda.toBase58()} 尚未初始化（data 长度应为 8），先执行: node examples/call-dev.js mint 100`
            );
        }
        const sourceBalance = sourceInfo.data.readBigUInt64BE(0);
        if (sourceBalance < amount) {
            throw new Error(`余额不足: ${sourceBalance} < ${amount}`);
        }
        keys = [
            { pubkey: payer.publicKey, isSigner: true, isWritable: true },    // from
            { pubkey: userPda, isSigner: false, isWritable: true },           // from PDA
            { pubkey: payer_2.publicKey, isSigner: false, isWritable: true }, // to（收款方无需签名）
            { pubkey: userPda2, isSigner: false, isWritable: true },          // to PDA
            ...tailKeys,
        ];
        data = Buffer.concat([Buffer.from([0x01]), amountBuf]);
        console.log(
            `Transfer ${amount}: ${userPda.toBase58()} -> ${userPda2.toBase58()}`
        );
    } else {
        throw new Error(`未知模式: ${mode}，只支持 mint / transfer`);
    }

    const instruction = new TransactionInstruction({ programId, keys, data });

    const { blockhash, lastValidBlockHeight } =
        await connection.getLatestBlockhash("confirmed");

    const transaction = new Transaction({
        recentBlockhash: blockhash,
        feePayer: payer.publicKey,
    }).add(instruction);

    // 只有 from / minter 需要签名，payer_2 只是收款方
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