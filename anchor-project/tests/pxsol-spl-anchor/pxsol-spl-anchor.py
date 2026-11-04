import argparse
import base64
import inspect
import json
import os
import random
import subprocess

import pxsol


def load_prikey(path='~/.config/solana/id.json'):
    k = json.load(open(os.path.expanduser(path)))[:32]
    n = int.from_bytes(bytes(k), 'big')
    alpha = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
    s = ''
    while n:
        n, r = divmod(n, 58)
        s = alpha[r] + s
    return '1' * (len(k) - len(bytes(k).lstrip(b'\0'))) + s


parser = argparse.ArgumentParser()
parser.add_argument('--net', type=str, choices=['develop', 'mainnet', 'testnet'], default='develop')
parser.add_argument('--prikey', type=str, default=None)
parser.add_argument('args', nargs='+')
args = parser.parse_args()

if args.net == 'develop':
    pxsol.config.current = pxsol.config.develop
if args.net == 'mainnet':
    pxsol.config.current = pxsol.config.mainnet
if args.net == 'testnet':
    pxsol.config.current = pxsol.config.testnet
pxsol.config.current.log = 1

user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey or load_prikey()))

# 程序 ID 沿用 pxsol-spl-anchor 仓库的部署地址，Anchor.toml 和 lib.rs 里必须一致。
prog_pubkey = pxsol.core.PubKey.base58_decode('21bhu54GetZ1EkmC9J1BkQvGAv37hxoZvcg3nuEy7r9h')

script_dir = os.path.dirname(os.path.abspath(__file__))
anchor_dir = os.path.dirname(script_dir)

# deploy 生成的 mint 密钥对保存在脚本目录下，供 airdrop 复用。
info_file = os.path.join(script_dir, 'pxsol-spl-anchor.info.json')

# 程序的 airdrop 使用 anchor_spl::token_2022，所以这里必须用 Token-2022 程序地址。
# pxsol.program.Token.pubkey 默认就指向 Token-2022。
token_program_pubkey = pxsol.program.Token.pubkey
ata_program_pubkey = pxsol.program.AssociatedTokenAccount.pubkey


def spl_create(decimals: int, metadata: dict):
    # pxsol 的 spl_create 有两代签名：
    #   1.x: spl_create(decimals, {'metadata': {...}})
    #   0.4/0.5: spl_create(name, symbol, uri, decimals)
    # 按实际安装版本的函数签名自动选择，避免写死版本。
    params = inspect.signature(user.spl_create).parameters
    if 'extension' in params:
        return user.spl_create(decimals, {'metadata': metadata})
    return user.spl_create(metadata['name'], metadata['symbol'], metadata['uri'], decimals)


def derive_pda(pubkey, seed: bytearray):
    # 1.x 的 derive_pda 返回 (pubkey, bump)，0.4/0.5 直接返回 pubkey。
    r = pubkey.derive_pda(seed)
    return r[0] if isinstance(r, tuple) else r


def spl_account(owner, mint):
    # ATA 派生规则：seeds = [owner, mint 所属 token program, mint]，程序地址是 ATA program。
    # 这里按 pxsol 内部同样的方式自己算，避免依赖 Wallet.spl_account（老版本没有这个方法）。
    info = pxsol.rpc.get_account_info(mint.base58(), {})
    host = pxsol.core.PubKey.base58_decode(info['owner'])
    seed = bytearray()
    seed.extend(owner.p)
    seed.extend(host.p)
    seed.extend(mint.p)
    return derive_pda(ata_program_pubkey, seed)


def info_save(k: str, v: str) -> None:
    info = {}
    if os.path.exists(info_file):
        with open(info_file) as f:
            info = json.load(f)
    info.setdefault(args.net, {})[k] = v
    with open(info_file, 'w') as f:
        json.dump(info, f, indent=4)


def info_load(k: str) -> str:
    if not os.path.exists(info_file):
        raise SystemExit(f'info not found: {info_file}, run "deploy" first')
    with open(info_file) as f:
        info = json.load(f)
    return info[args.net][k]


def call(c: str):
    return subprocess.run(c, check=True, shell=True, cwd=anchor_dir)


def deploy():
    # 创建 Token-2022 mint
    print('main: create mint')
    pubkey_mint = spl_create(9, {
        'name': 'PXSOL',
        'symbol': 'PXS',
        'uri': 'https://raw.githubusercontent.com/mohanson/pxsol/refs/heads/master/res/pxs.json',
    })
    print(f'main: create mint pubkey={pubkey_mint.base58()}')
    info_save('pubkey_mint', pubkey_mint.base58())

    # 给自己 mint 足量代币
    print(f'main: mint 100000000 for {user.pubkey.base58()}')
    user.spl_mint(pubkey_mint, user.pubkey, 100000000 * 10**9)

    # 部署链上程序。localnet 下需要与 declare_id 匹配的程序 keypair，
    # 即 target/deploy/pxsol_spl_anchor-keypair.json，并且 Anchor.toml 里登记同一个地址。
    call('anchor deploy')

    # 把代币打进程序金库（PDA 的 ATA）
    pubkey_mana_seed = bytearray()
    pubkey_mana_auth = derive_pda(prog_pubkey, pubkey_mana_seed)
    user.spl_transfer(pubkey_mint, pubkey_mana_auth, 100000000 * 10**9)
    print(f'main: fund pool {pubkey_mana_auth.base58()}')


def genuser():
    print('main: random user')
    user_new = pxsol.wallet.Wallet(pxsol.core.PriKey(bytearray(random.randbytes(32))))
    print(f'main: random user prikey={user_new.prikey}')
    print(f'main: random user pubkey={user_new.pubkey}')
    print('main: request sol airdrop')
    txid = pxsol.rpc.request_airdrop(user_new.pubkey.base58(), 1_000_000_000, {})
    print(f'main: request sol airdrop txid={txid}')
    pxsol.rpc.wait([txid])
    print('main: request sol airdrop done')


def airdrop():
    pubkey_mint = pxsol.core.PubKey.base58_decode(info_load('pubkey_mint'))
    pubkey_mana = prog_pubkey
    pubkey_mana_seed = bytearray()

    # 金库授权地址 = 程序 PDA（seeds = [b""]），余额保存在它的 ATA 里。
    pubkey_mana_auth = derive_pda(pubkey_mana, pubkey_mana_seed)
    pubkey_mana_spla = spl_account(pubkey_mana_auth, pubkey_mint)

    rq = pxsol.core.Requisition(pubkey_mana, [], bytearray())
    # 与 Airdrop<'info> 的字段顺序一致：
    # user, user_spla, mana, mana_auth, mana_spla, mint, token_program, associated_token_program, system_program
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))             # user: mut + signer
    rq.account.append(pxsol.core.AccountMeta(spl_account(user.pubkey, pubkey_mint), 1))  # user_spla: mut，不存在时由 init_if_needed 创建
    rq.account.append(pxsol.core.AccountMeta(pubkey_mana, 0))             # mana: 程序自身
    rq.account.append(pxsol.core.AccountMeta(pubkey_mana_auth, 0))        # mana_auth: PDA
    rq.account.append(pxsol.core.AccountMeta(pubkey_mana_spla, 1))        # mana_spla: 金库 ATA
    rq.account.append(pxsol.core.AccountMeta(pubkey_mint, 0))             # mint
    rq.account.append(pxsol.core.AccountMeta(token_program_pubkey, 0))    # token_program: Token-2022
    rq.account.append(pxsol.core.AccountMeta(ata_program_pubkey, 0))      # associated_token_program
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    # Anchor instruction discriminator:
    # sha256("global:airdrop") 的前 8 字节
    rq.data = bytearray([113, 173, 36, 238, 38, 152, 22, 117])

    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    print('main: request spl airdrop')
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    tlog = pxsol.rpc.get_transaction(txid, {})
    if tlog and tlog.get('meta', {}).get('err') is not None:
        print('main: airdrop failed', tlog['meta']['err'])
    for e in (tlog or {}).get('meta', {}).get('logMessages', []):
        print(e)
    # 直接查 ATA 余额，不依赖 Wallet.spl_balance（老版本同样可能没有）。
    splcnt = pxsol.rpc.get_token_account_balance(spl_account(user.pubkey, pubkey_mint).base58(), {})['value']
    print(f'main: request spl airdrop done recv={int(splcnt["amount"]) / 10**splcnt["decimals"]}')


if __name__ == '__main__':
    eval(f'{args.args[0]}()')
