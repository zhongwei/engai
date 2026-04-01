import { Badge } from '@/components/ui/badge'

const colors: Record<number, string> = {
  0: 'bg-gray-200 text-gray-700',
  1: 'bg-red-200 text-red-700',
  2: 'bg-orange-200 text-orange-700',
  3: 'bg-yellow-200 text-yellow-700',
  4: 'bg-green-200 text-green-700',
  5: 'bg-blue-200 text-blue-700',
}

const labels: Record<number, string> = {
  0: 'New',
  1: 'Again',
  2: 'Hard',
  3: 'Difficult',
  4: 'Good',
  5: 'Perfect',
}

export default function FamiliarityBadge({ level }: { level: number }) {
  const l = Math.max(0, Math.min(5, level))
  return (
    <Badge className={colors[l]} variant="secondary">
      {labels[l]}
    </Badge>
  )
}
