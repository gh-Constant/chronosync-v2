FROM node:20-alpine

WORKDIR /app

# Installer pnpm
RUN npm install -g pnpm

# Copier le package.json racine et le fichier de workspace
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# Copier les package.json des projets
# Cela permet de n'installer que les dépendances du backend
COPY apps/backend/package.json ./apps/backend/
COPY packages/shared-types/package.json ./packages/shared-types/

# Installer uniquement les dépendances du backend
RUN pnpm install --filter backend --prod

# Copier le code source du backend et le package partagé
COPY apps/backend ./apps/backend
COPY packages/shared-types ./packages/shared-types

# Copier le .env.example pour référence (le .env sera injecté au runtime)
COPY .env.example .

WORKDIR /app/apps/backend

# Construire le projet
RUN pnpm run build

# Exposer le port
EXPOSE 3000

# Lancer l'application en production
CMD ["pnpm", "run", "start"]