import { PrismaClient } from '@prisma/client';

// Singleton pour le client Prisma
const prisma = new PrismaClient({
    log: process.env.NODE_ENV === 'development' ? ['query', 'info', 'warn', 'error'] : ['error'],
});

export default prisma;