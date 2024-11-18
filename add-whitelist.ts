import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultClaim } from "./target/types/vault_claim";

async function main() {
    // Configure the client
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.VaultClaim as Program<VaultClaim>;

    const walletsToWhitelist = [
        "4B7uc4LDAucp47fq1YUfNiQUJ6GchSSAbgWWx3sB6dgq",
        "4Fvmfhw6z31ScqJrz9pUe6y6cqNcEXRvaBzFLNQjTtwU",
        "95nNZwgjNE95eZyUevK987UAim8qgwvfeLkqGRC7y2sD"
    ];

    // 1 USDC = 1_000_000 (6 decimals)
    const claimAmount = new anchor.BN(1_000_000);

    const vaultAddress = new anchor.web3.PublicKey("Gg74S94SW1aHa6vVqEHyTmk2ezaAUdagXJXBSh7uP9yw");  // Replace with your vault address

    for (const walletAddress of walletsToWhitelist) {
        try {
            const tx = await program.methods
                .addToWhitelist(new anchor.web3.PublicKey(walletAddress), claimAmount)
                .accounts({
                    vault: vaultAddress,
                    user: program.provider.publicKey,
                })
                .rpc();
            console.log(`Added ${walletAddress} to whitelist. Tx: ${tx}`);
        } catch (e) {
            console.error(`Failed to add ${walletAddress}:`, e);
        }
    }
}

main().then(
    () => process.exit(0),
    (err) => {
        console.error(err);
        process.exit(1);
    }
);
