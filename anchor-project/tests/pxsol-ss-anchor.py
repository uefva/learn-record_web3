import argparse
import base64
import pxsol
import json, os

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

user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey or load_prikey()))
prog_pubkey = pxsol.core.PubKey.base58_decode('2sPwcBjsWBRnLZEShoYWCRuY8PAat9HGGsUHhjf3VXZK')
data_pubkey = prog_pubkey.derive_pda(bytearray(b'data') + user.pubkey.p)

# init_mint 生成的 mint 密钥对保存在脚本目录下的 mint.json，供 init_token 复用。
mint_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'mint.json')

# Anchor 的 anchor_spl::token::Token 是经典 SPL Token 程序，而 pxsol.program.Token.pubkey 默认指向 Token-2022，
# 所以这里必须显式使用 pubkey_2020。
token_program_pubkey = pxsol.program.Token.pubkey_2020
ata_program_pubkey = pxsol.program.AssociatedTokenAccount.pubkey


def load_mint():
    if not os.path.exists(mint_file):
        raise SystemExit(f'mint keypair not found: {mint_file}, run "init_mint" first')
    with open(mint_file) as f:
        return pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(json.load(f)['prikey']))


def init():
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.data = bytearray().join([
        bytearray([220, 59, 207, 236, 108, 250, 47, 100]),
    ])
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def update():
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.data = bytearray().join([
        bytearray([219, 200, 88, 176, 158, 63, 253, 127]),
        len(args.args[1].encode()).to_bytes(4, 'little'),
        args.args[1].encode(),
    ])
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)

def append():
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.data = bytearray().join([
        bytearray([149, 120, 18, 222, 236, 225, 88, 203]),
        len(args.args[1].encode()).to_bytes(4, 'little'),
        args.args[1].encode(),
    ])
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def init_mint():
    # Anchor 的 init 约束在 mint 没有 seeds 时会走 system_program::create_account，
    # 因此 mint 必须是新生成的 keypair，并在交易中签名。
    mint = pxsol.wallet.Wallet(pxsol.core.PriKey.random())
    with open(mint_file, 'w') as f:
        json.dump({'prikey': mint.prikey.base58(), 'pubkey': mint.pubkey.base58()}, f)

    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    # 与 CreateMint 的字段顺序一致：signer, mint, system_program, token_program
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))  # signer: mut + signer
    rq.account.append(pxsol.core.AccountMeta(mint.pubkey, 3))  # mint: init，需要签名
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(token_program_pubkey, 0))
    # Anchor instruction discriminator:
    # sha256("global:init_mint") 的前 8 字节
    rq.data = bytearray([
        126, 176, 233, 16, 66, 117, 209, 125
    ])

    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    # 签名顺序必须与交易里的签名者顺序一致：付费账户在前，mint 在后
    tx.sign([user.prikey, mint.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])

    print('mint:', mint.pubkey.base58())
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def init_token():
    mint = load_mint()
    # ATA 派生：seeds = [owner, token program, mint]，程序地址为 associated token program
    token_pubkey = ata_program_pubkey.derive_pda(user.pubkey.p + token_program_pubkey.p + mint.pubkey.p)

    # CreateToken 里 token 账户只有 associated_token::* 约束、没有 init，
    # Anchor 只会校验不会创建，ATA 不存在会直接 AccountNotInitialized。
    # 所以在同一笔交易里先创建 ATA，再调用 init_token。
    rq_ata = pxsol.core.Requisition(
        ata_program_pubkey, [],
        pxsol.program.AssociatedTokenAccount.create_idempotent(),
    )
    rq_ata.account.append(pxsol.core.AccountMeta(user.pubkey, 3))         # 0. 付费账户
    rq_ata.account.append(pxsol.core.AccountMeta(token_pubkey, 1))        # 1. 待创建的 ATA
    rq_ata.account.append(pxsol.core.AccountMeta(user.pubkey, 0))         # 2. ATA 的 owner
    rq_ata.account.append(pxsol.core.AccountMeta(mint.pubkey, 0))         # 3. mint
    rq_ata.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq_ata.account.append(pxsol.core.AccountMeta(token_program_pubkey, 0))

    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    # 与 CreateToken 的字段顺序一致：
    # signer, mint, token, system_program, token_program, associated_token_program
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))  # signer: mut + signer
    rq.account.append(pxsol.core.AccountMeta(mint.pubkey, 0))  # mint: readonly
    rq.account.append(pxsol.core.AccountMeta(token_pubkey, 1))  # token: mut
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(token_program_pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(ata_program_pubkey, 0))
    # Anchor instruction discriminator:
    # sha256("global:init_token") 的前 8 字节
    rq.data = bytearray([
        9, 187, 206, 103, 139, 126, 246, 181
    ])

    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq_ata, rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])

    print('mint:', mint.pubkey.base58())
    print('token:', token_pubkey.base58())
    r = pxsol.rpc.get_transaction(txid, {})
    if r and r.get('meta', {}).get('err') is not None:
        print('Init token failed:', r['meta']['err'])
    for e in (r or {}).get('meta', {}).get('logMessages', []):
        print(e)


def load():
    info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})
    print(base64.b64decode(info['data'][0])[8 + 32 + 1 + 4:].decode())


def close():
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())

    # 与 Anchor 的 Close<'info> 账户顺序一致
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))

    # Anchor instruction discriminator:
    # sha256("global:close") 的前 8 字节
    rq.data = bytearray([
        98, 165, 201, 177, 108, 65, 206, 96
    ])

    tx = pxsol.core.Transaction.requisition_decode(
        user.pubkey, [rq]
    )
    tx.message.recent_blockhash = pxsol.base58.decode(
        pxsol.rpc.get_latest_blockhash({})['blockhash']
    )
    tx.sign([user.prikey])

    txid = pxsol.rpc.send_transaction(
        base64.b64encode(tx.serialize()).decode(), {}
    )
    pxsol.rpc.wait([txid])

    r = pxsol.rpc.get_transaction(txid, {})
    print('Transaction:', txid)

    if r and r.get('meta', {}).get('err') is not None:
        print('Close failed:', r['meta']['err'])

    for e in (r or {}).get('meta', {}).get('logMessages', []):
        print(e)



if __name__ == '__main__':
    eval(f'{args.args[0]}()')