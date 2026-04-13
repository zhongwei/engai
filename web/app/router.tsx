import { createRouter, createRootRoute, createRoute, lazyRouteComponent } from '@tanstack/react-router'
import RootLayout from './root'

const rootRoute = createRootRoute({
  component: RootLayout,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: lazyRouteComponent(() => import('./routes/_index')),
})

const vocabularyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/vocabulary',
  component: lazyRouteComponent(() => import('./routes/vocabulary')),
})

const reviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/review',
  component: lazyRouteComponent(() => import('./routes/review')),
})

const readingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/readings',
  component: lazyRouteComponent(() => import('./routes/readings')),
})

const chatRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/chat',
  component: lazyRouteComponent(() => import('./routes/chat')),
})

const wordRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/words/$word',
  component: lazyRouteComponent(() => import('./routes/words.$word')),
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  vocabularyRoute,
  reviewRoute,
  readingsRoute,
  chatRoute,
  wordRoute,
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
