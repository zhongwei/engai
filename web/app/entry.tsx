import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider } from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { router } from './router'

const queryClient = new QueryClient()

const rootElement = document.getElementById('root')!
const root = import.meta.hot?.data.root ?? createRoot(rootElement)
if (import.meta.hot) {
  import.meta.hot.data.root = root
  import.meta.hot.accept()
}

root.render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
)
