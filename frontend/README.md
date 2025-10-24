# PriceCrowd Frontend

Single Page Application scaffolded with Vue 3 + Vite + Tailwind CSS.

## Scripts

- `npm run dev` — start dev server
- `npm run build` — build for production
- `npm run preview` — preview production build

## Setup

1. Install dependencies:
   - npm: `npm install`
   - pnpm: `pnpm install`
   - yarn: `yarn`
2. Start development server: `npm run dev`

Tailwind is configured in `tailwind.config.cjs` and `postcss.config.cjs`. Styles are imported via `src/index.css`.

Routes are defined in `src/router/index.ts`. Add new views under `src/views` and register them in the router.
