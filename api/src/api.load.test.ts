import { rest } from 'msw';
import { createApi } from './index';
import { server } from './mocks';

function percentile(values: number[], ratio: number): number {
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.ceil(sorted.length * ratio) - 1);
  return sorted[index];
}

describe('API load', () => {
  it('handles concurrent read traffic within a reasonable in-memory latency budget', async () => {
    const api = createApi('http://localhost:3000');

    server.use(
      rest.get('/api/trades', (_req, res, ctx) => res(ctx.delay(25), ctx.json([{ id: '1', seller: 'A', buyer: 'B', amount: '1', status: 'created', timestamp: '2024-03-25T10:30:00Z' }]))),
      rest.get('/api/events', (_req, res, ctx) =>
        res(
          ctx.delay(25),
          ctx.json([
            {
              id: '1',
              type: 'trade_created',
              tradeId: '1',
              timestamp: '2024-03-25T10:30:00Z',
              data: {},
            },
          ])
        )
      )
    );

    const requests = Array.from({ length: 20 }, (_, index) => async () => {
      const startedAt = Date.now();
      if (index % 2 === 0) {
        await api.trades.getTrades(1, 0);
      } else {
        await api.events.getEvents(1);
      }
      return Date.now() - startedAt;
    });

    const totalStartedAt = Date.now();
    const durations = await Promise.all(requests.map((run) => run()));
    const totalDuration = Date.now() - totalStartedAt;

    expect(durations).toHaveLength(20);
    expect(percentile(durations, 0.95)).toBeLessThan(1000);
    expect(totalDuration).toBeLessThan(3000);
  });
});
