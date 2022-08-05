import { deserialize, DeserializeOptions } from 'bson'
import { readSync } from 'node:fs'
import { Buffer } from 'node:buffer'

function main (): void {
  const buffer = Buffer.alloc(1024 * 1024 * 1024)
  const size = readSync(0, buffer)

  const base64 = buffer.toString('utf-8', 0, size)
  const bson = Buffer.from(base64, 'base64')

  const options: DeserializeOptions = {
    allowObjectSmallerThanBufferSize: true
  }
  const document = deserialize(bson, options).data
  document.sort(sortRecord)
  prettify(document)
  console.table(document)
}

function prettify (data: any[]): void {
  for (const record of data) {
    const change = record.change
    if (typeof (change) === 'number') {
      record.change = `${(change * 100).toFixed(2)}%`
    }

    const fromDate = record.from_date
    if (typeof (fromDate) === 'number') {
      record.from_date = formatDate(fromDate)
    }

    const toDate = record.to_date
    if (typeof (toDate) === 'number') {
      record.to_date = formatDate(toDate)
    }
  }
}

function sortRecord (a: any, b: any): number {
  if (typeof (a.change) === 'number') {
    if (typeof (b.change) === 'number') {
      return a.change - b.change
    } else {
      return 1
    }
  } else {
    return -1
  }
}

function formatDate (date: number): string {
  return new Date(date).toLocaleDateString()
}

main()
