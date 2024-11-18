it("Initializes the vault", async () => {
    const initAmount = new anchor.BN(10_000);
  
    await program.methods
      .initialize(initAmount)
      .accounts({
        vault: vault.publicKey,
        vaultTokenAccount: vaultTokenAccount,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vault])
      .rpc();
  
    // Fetch and cast vault account correctly
    const vaultAccount = await program.account.vault.fetch(vault.publicKey) as VaultAccount;
  
    assert.ok(vaultAccount.amount.eq(initAmount), "Vault amount mismatch");
    assert.equal(vaultAccount.payer.toBase58(), provider.wallet.publicKey.toBase58(), "Payer mismatch");
  });
  
  it("Adds a user to the whitelist", async () => {
    const whitelistWallet = Keypair.generate().publicKey;
  
    await program.methods
      .addToWhitelist(whitelistWallet, claimableAmount)
      .accounts({
        vault: vault.publicKey,
        user: provider.wallet.publicKey,
      })
      .signers([])
      .rpc();
  
    // Verify the whitelist addition
    const vaultAccount = await program.account.vault.fetch(vault.publicKey) as VaultAccount;
    const whitelist = vaultAccount.whitelist;
  
    // Check if the wallet is in the whitelist
    const entry = whitelist.find(
      (entry) => entry.address.toBase58() === whitelistWallet.toBase58()
    );
    assert.ok(entry, "Wallet is not in the whitelist");
    assert.ok(entry?.amount.eq(claimableAmount), "Claimable amount does not match");
  });
  
  it("Allows whitelisted user to claim tokens", async () => {
    const claimAmount = new anchor.BN(500);
    const whitelistedUser = Keypair.generate();
  
    // Add whitelisted user for claim test
    await program.methods
      .addToWhitelist(whitelistedUser.publicKey, claimAmount)
      .accounts({
        vault: vault.publicKey,
        user: provider.wallet.publicKey,
      })
      .signers([])
      .rpc();
  
    // Claim tokens
    await program.methods
      .claim()
      .accounts({
        vault: vault.publicKey,
        vaultTokenAccount: vaultTokenAccount,
        userTokenAccount: userTokenAccount,
        claimant: whitelistedUser.publicKey,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      })
      .signers([whitelistedUser])
      .rpc();
  
    // Verify token balance after claim
    const userTokenBalance = await getTokenAccountBalance(provider, userTokenAccount);
    assert.equal(userTokenBalance, claimAmount.toNumber(), "User token balance mismatch");
  
    // Check that vault balance has decreased
    const vaultAccount = await program.account.vault.fetch(vault.publicKey) as VaultAccount;
    const remainingVaultBalance = 10_000 - claimAmount.toNumber();
    assert.ok(vaultAccount.amount.eq(new anchor.BN(remainingVaultBalance)), "Vault balance mismatch");
  });
  