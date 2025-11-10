import express from 'express';
import { prisma } from './config/database.js';

const app = express();
app.use(express.json());

app.post('/usage', async (req, res) => {
  const { window, duration } = req.body;
  // TODO: Get user from request
  const userId = 'some-user-id';

  await prisma.usageLog.create({
    data: {
      time: new Date(),
      user_id: userId,
      app_name: window,
      app_identifier: window,
      device_type: 'desktop',
      duration_seconds: Math.round(duration),
    },
  });

  res.sendStatus(200);
});

app.get('/', (req, res) => {
  res.send('Hello, world!');
});

export { app };