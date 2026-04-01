import { useStats } from '@/features/stats/queries'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { BookOpen, MessageSquare, RefreshCw, CheckCircle } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

export default function Dashboard() {
  const { data: stats, isLoading } = useStats()

  const cards: { title: string; value: number | undefined; icon: LucideIcon; color: string }[] = [
    { title: 'Words', value: stats?.word_count, icon: BookOpen, color: 'text-blue-500' },
    { title: 'Phrases', value: stats?.phrase_count, icon: MessageSquare, color: 'text-purple-500' },
    { title: 'Pending Reviews', value: stats?.pending_reviews, icon: RefreshCw, color: 'text-orange-500' },
    { title: 'Reviewed Today', value: stats?.reviewed_today, icon: CheckCircle, color: 'text-green-500' },
  ]

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Dashboard</h1>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {cards.map(({ title, value, icon: Icon, color }) => (
          <Card key={title}>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">{title}</CardTitle>
              <Icon className={`h-5 w-5 ${color}`} />
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <div className="h-8 bg-muted animate-pulse rounded" />
              ) : (
                <div className="text-3xl font-bold">{value ?? 0}</div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
