老大，我先把脚本当前内容读一遍（你刚改过），再给你完整用法。老大，脚本是"哪条命令就执行哪个函数"的写法，`deploy` / `genuser` / `airdrop` 三个动作。当前你这边已经 `anchor build` 成功、`target/deploy/pxsol_spl_anchor.so` 也在，说明程序能编了；脚本依赖 `pxsol` 这个 Python 库（你现有的 `pxsol-ss-anchor.py` 已经用了它，环境里应该装过，没有就 `pip install pxsol`）。

**完整流程（在 WSL 里，默认 `--net develop` 即本地 127.0.0.1:8899）**

1. 先起本地验证节点（你的 `Anchor.toml` 是 `skip_local_validator = true`，anchor 不会自己拉节点）：
```bash
solana-test-validator -l /tmp/solana-ledger
```
2. 部署程序（和 Anchor.toml 里 `[scripts]` 无关，直接手跑）：
```bash
cd /mnt/e/fengkai/my_project/learn-record_web3/anchor-project
anchor deploy
```
3. 初始化代币和资金池。这一步会：建一个 9 位小数的 Token-2022 mint → 给自己 mint 1 亿枚 → 调一次 `anchor deploy` → 把 1 亿枚打进程序 PDA 的金库 ATA，并把 mint 地址写进 `tests/pxsol-spl-anchor.info.json`：
```bash
python tests/pxsol-spl-anchor/pxsol-spl-anchor.py deploy
```
4. 造一个领币用户并给它 1 SOL（本地 faucet）：
```bash
python tests/pxsol-spl-anchor/pxsol-spl-anchor.py genuser
# 输出里有一行 random user prikey=xxxx，记下来
```
5. 用那个私钥领空投，固定领 5 个代币：
```bash
python tests/pxsol-spl-anchor/pxsol-spl-anchor.py --prikey <上一步的私钥> airdrop
```
成功时最后一行是 `main: request spl airdrop done recv=5.0`，中间会把交易日志打出来。

**几个容易踩的点**

- `--net` 三个命令必须一致，默认 `develop` 就是本地；`info.json` 也按 net 分开存。换 net 不重新 deploy 会报 `info not found`。
- 第 5 步的用户账户里必须有 SOL，否则 `init_if_needed` 建 ATA 时付不出租金，所以要先 `genuser`。
- 空投没有次数限制，同一个私钥可以反复领，这是仓库本身的设计。
- `deploy` 里那句 `call('anchor deploy')` 会部署 workspace 里**所有**程序（包括旧的 `anchor_project`），不是只部署新程序。
- 脚本第 92-93 行还留着"AQkW 没私钥、本地会失败"的旧注释，现在已经不成立了，你要我清理的话我顺手改。

另外 `Anchor.toml` 里 `pxsol_spl_anchor` 仍是 `AQkWeJJm...`，而 `declare_id!` 和 keypair 已是 `21bhu54...`。你这次 deploy 能过，是因为 Anchor 部署用的是 `target/deploy` 下的 keypair，但正常做法这两处要一致，建议同步一下——要改我就改，等你一句话。