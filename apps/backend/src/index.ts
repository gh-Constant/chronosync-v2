import { app } from './app.js';
import { config } from './config';

const port = config.port;

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});