import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { Event, Trade } from './models';

const initialTrades: Trade[] = [
  {
    id: '1',
    seller: 'GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE',
    buyer: 'GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD',
    amount: '100.50',
    status: 'completed',
    timestamp: '2024-03-25T10:30:00Z',
  },
];

const initialEvents: Event[] = [
  {
    id: '1',
    type: 'trade_created',
    tradeId: '1',
    timestamp: '2024-03-25T10:30:00Z',
    data: {},
  },
];
import { tradeHandlers } from './mocks/handlers/trades';
import { eventHandlers } from './mocks/handlers/events';
import { blockchainHandlers } from './mocks/handlers/blockchain';

let mockTrades: Trade[] = [];
let mockEvents: Event[] = [];
let txCounter = 0;

function cloneTrades(): Trade[] {
  return initialTrades.map((trade) => ({ ...trade }));
}

function cloneEvents(): Event[] {
  return initialEvents.map((event) => ({ ...event, data: { ...event.data } }));
}

function nextTxHash(): string {
  txCounter += 1;
  return `0xtx${txCounter.toString().padStart(4, '0')}`;
}

export function resetMockData() {
  mockTrades = cloneTrades();
  mockEvents = cloneEvents();
  txCounter = 0;
}

resetMockData();

export const handlers = [
  // Trades
  rest.get('/api/trades', (req, res, ctx) => {
    const limit = req.url.searchParams.get('limit') || '50';
    return res(ctx.json(mockTrades.slice(0, parseInt(limit))));
  }),

  rest.get('/api/trades/:id', (req, res, ctx) => {
    const trade = mockTrades.find((t) => t.id === req.params.id);
    return trade ? res(ctx.json(trade)) : res(ctx.status(404), ctx.json({ error: 'Not found' }));
  }),

  rest.post('/api/trades', (req, res, ctx) => {
    const newTrade = { id: String(mockTrades.length + 1), ...req.body };
    mockTrades.push(newTrade);
    return res(ctx.status(201), ctx.json(newTrade));
  }),

  rest.patch('/api/trades/:id', (req, res, ctx) => {
    const trade = mockTrades.find((t) => t.id === req.params.id);
    if (!trade) return res(ctx.status(404), ctx.json({ error: 'Not found' }));
    Object.assign(trade, req.body);
    return res(ctx.json(trade));
  }),

  rest.delete('/api/trades/:id', (req, res, ctx) => {
    const index = mockTrades.findIndex((t) => t.id === req.params.id);
    if (index === -1) return res(ctx.status(404), ctx.json({ error: 'Not found' }));
    mockTrades.splice(index, 1);
    return res(ctx.status(204));
  }),

  // Events
  rest.get('/api/events', (req, res, ctx) => {
    const limit = req.url.searchParams.get('limit') || '100';
    const tradeId = req.url.searchParams.get('tradeId');
    const filtered = tradeId ? mockEvents.filter((event) => event.tradeId === tradeId) : mockEvents;
    return res(ctx.json(filtered.slice(0, parseInt(limit))));
  }),

  rest.get('/api/events/trade/:tradeId', (req, res, ctx) => {
    const events = mockEvents.filter((e) => e.tradeId === req.params.tradeId);
    return res(ctx.json(events));
  }),

  rest.get('/api/events/:id', (req, res, ctx) => {
    const event = mockEvents.find((e) => e.id === req.params.id);
    return event ? res(ctx.json(event)) : res(ctx.status(404), ctx.json({ error: 'Not found' }));
  }),

  // Blockchain
  rest.post('/api/blockchain/fund', (req, res, ctx) => {
    return res(ctx.json({ txHash: nextTxHash() }));
  }),

  rest.post('/api/blockchain/complete', (req, res, ctx) => {
    return res(ctx.json({ txHash: nextTxHash() }));
  }),

  rest.post('/api/blockchain/resolve', (req, res, ctx) => {
    return res(ctx.json({ txHash: nextTxHash() }));
  }),

  rest.get('/api/blockchain/tx/:txHash', (req, res, ctx) => {
    return res(ctx.json({ status: 'confirmed', confirmed: true }));
  }),
  ...tradeHandlers,
  ...eventHandlers,
  ...blockchainHandlers,
];
