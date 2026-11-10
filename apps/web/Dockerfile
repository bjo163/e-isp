FROM node:22.16.0-alpine
WORKDIR /app
COPY package.json ./
COPY apps/web/package.json ./apps/web/package.json
RUN npm install --no-package-lock
COPY apps/web ./apps/web
WORKDIR /app/apps/web
ENV NODE_ENV=production
RUN npm run build
EXPOSE 3000
CMD ["npm", "run", "start"]
