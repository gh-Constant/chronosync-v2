import app from './app';
import dotenv from 'dotenv';

// Charger les variables d'environnement depuis la racine
dotenv.config({ path: '../../.env' });

const port = process.env.PORT || 3000;

app.listen(port, () => {
    console.log(`[Backend] Serveur démarré sur http://localhost:${port}`);
});