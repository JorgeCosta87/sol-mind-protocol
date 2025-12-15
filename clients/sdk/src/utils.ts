import {
  type TransactionSigner,
  type SolanaClient,
  type Instruction,
} from 'gill';
import {
  createTransaction,
  signTransactionMessageWithSigners,
  getSignatureFromTransaction,
} from 'gill';

export async function sendTransaction(
  client: SolanaClient,
  payer: TransactionSigner,
  instructions: Instruction[]
): Promise<string> {
  const { value: latestBlockhash } = await client.rpc.getLatestBlockhash().send();

  const transaction = createTransaction({
    feePayer: payer,
    instructions,
    latestBlockhash,
  });

  const signedTransaction = await signTransactionMessageWithSigners(transaction);

  await client.sendAndConfirmTransaction(signedTransaction);

  return getSignatureFromTransaction(signedTransaction);
}

