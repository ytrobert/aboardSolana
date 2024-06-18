# aboardSolana
## deployment address
perpetual: Geoia2xs6aEdRKL3AWCgxKRkhyWZEQmafvFsvm4U3UX9

USDC(decimal 6): EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

SOL(decimal 9)

perpetual account(config): 4T2G7uPEYnjDBkQ9MyeDMmPMHQLRnvqRdpgRyMKtS8st

program token account (vault): 
  
  USDC: B3G7Fb52tpqM2pQPtxU6gkUSFGNaruChhQTp5iBaGAUy
  
  SOL: GcbY3wxHati9x7xhKaMf33LuFfvk4Ch2KQWyWMwNQoiw

deployer: 5PCYSM6kf1iZnPYGApAFYb1FPexRGqEgVDTodeW8nMoE (solana playground)

admin: 5PCYSM6kf1iZnPYGApAFYb1FPexRGqEgVDTodeW8nMoE

gateway: 2FrnEJz4eY3qtaERZCQBouS9cuSCQ9kUy2Z2CWis4LMf

signer: 

secp256k1_pubkey: [155, 184, 235, 12, 127, 133, 226, 211, 136, 228, 200, 65, 8, 
                   186, 159, 244, 170, 38, 122, 0, 1, 139, 144, 46, 240, 4, 145, 
                   135, 252, 93, 117, 158, 193, 119, 111, 188, 160, 71, 224, 13, 
                   135, 243, 224, 66, 15, 44, 0, 254, 84, 243, 220, 122, 0, 212, 
                   12, 139, 61, 5, 138, 90, 179, 162, 177, 55]
## Account
  1. perpetual account: configuration of Perpetual Exchange, is unique, pda from program_id
     including admin: only admin can update the account
               token_map: accountType, symbol, mint(token), program_token_account(vault, owner is program_id)                               
  2. user account: user information, is unique, pda from user publickey & program_id
     including use publickey: signer == user account
               withdraw_id: >
