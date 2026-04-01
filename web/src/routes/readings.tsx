import { createFileRoute } from '@tanstack/react-router'
import Reading from '@/pages/Reading'

export const Route = createFileRoute('/readings')({
  component: Reading,
})
