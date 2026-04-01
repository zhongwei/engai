import { createFileRoute } from '@tanstack/react-router'
import Vocabulary from '@/pages/Vocabulary'

export const Route = createFileRoute('/vocabulary')({
  component: Vocabulary,
})
