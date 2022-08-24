import { env } from 'node:process'
import { Toy } from './toy'

async function main (): Promise<void> {
  // IBKR Gateway uses a self-signed TLS certificate
  env.NODE_TLS_REJECT_UNAUTHORIZED = '0'
  return await new Toy().run()
}

await main()

// Dummy export to make this file an ES module, so that we can use top-level await.
export {}
