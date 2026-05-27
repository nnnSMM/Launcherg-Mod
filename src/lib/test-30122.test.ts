import { describe, it } from 'vitest';
import { getWorkByScrape } from './scrapeWork';

describe('test 30122', () => {
  it('fetches 30122', async () => {
    const work = await getWorkByScrape(30122);
    console.log("DESCRIPTION:", work.description);
  }, 30000);
});
