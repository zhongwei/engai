import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { apiFetch } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import FamiliarityBadge from '@/components/FamiliarityBadge'
import { Plus } from 'lucide-react'

interface Word {
  word: string
  meaning: string
  phonetic?: string
  familiarity: number
}

interface Phrase {
  id: number
  phrase: string
  meaning: string
  familiarity: number
}

interface WordsResponse {
  words: Word[]
}

interface PhrasesResponse {
  phrases: Phrase[]
}

export default function Vocabulary() {
  const [search, setSearch] = useState('')

  const { data: wordsResp, isLoading: wordsLoading } = useQuery<WordsResponse>({
    queryKey: ['words'],
    queryFn: () => apiFetch<WordsResponse>('/words'),
  })

  const { data: phrasesResp, isLoading: phrasesLoading } = useQuery<PhrasesResponse>({
    queryKey: ['phrases'],
    queryFn: () => apiFetch<PhrasesResponse>('/phrases'),
  })

  const words = wordsResp?.words ?? []
  const phrases = phrasesResp?.phrases ?? []

  const filteredWords = words.filter(w =>
    w.word.toLowerCase().includes(search.toLowerCase()) ||
    w.meaning.toLowerCase().includes(search.toLowerCase())
  )

  const filteredPhrases = phrases.filter(p =>
    p.phrase.toLowerCase().includes(search.toLowerCase()) ||
    p.meaning.toLowerCase().includes(search.toLowerCase())
  )

  return (
    <div className="p-6 flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold">Vocabulary</h1>
        <Button variant="outline" size="sm">
          <Plus className="h-4 w-4 mr-1" />
          Add Word
        </Button>
      </div>
      <Input
        placeholder="Search words and phrases..."
        value={search}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearch(e.target.value)}
        className="mb-4 max-w-md"
      />
      <Tabs defaultValue="words" className="flex-1 flex flex-col min-h-0">
        <TabsList>
          <TabsTrigger value="words">Words ({filteredWords.length})</TabsTrigger>
          <TabsTrigger value="phrases">Phrases ({filteredPhrases.length})</TabsTrigger>
        </TabsList>
        <TabsContent value="words" className="flex-1 mt-2 min-h-0">
          <ScrollArea className="h-full">
            {wordsLoading ? (
              <div className="space-y-3 p-2">
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="h-16 bg-muted animate-pulse rounded-lg" />
                ))}
              </div>
            ) : filteredWords.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">No words found</p>
            ) : (
              <div className="space-y-2 p-2">
                {filteredWords.map(w => (
                  <Link key={w.word} to={`/words/${encodeURIComponent(w.word)}`}>
                    <Card className="hover:bg-accent transition-colors cursor-pointer">
                      <CardContent className="flex items-center justify-between p-3">
                        <div className="min-w-0">
                          <div className="font-medium">{w.word}</div>
                          {w.phonetic && <div className="text-sm text-muted-foreground">{w.phonetic}</div>}
                          <div className="text-sm text-muted-foreground truncate">{w.meaning}</div>
                        </div>
                        <FamiliarityBadge level={w.familiarity} />
                      </CardContent>
                    </Card>
                  </Link>
                ))}
              </div>
            )}
          </ScrollArea>
        </TabsContent>
        <TabsContent value="phrases" className="flex-1 mt-2 min-h-0">
          <ScrollArea className="h-full">
            {phrasesLoading ? (
              <div className="space-y-3 p-2">
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="h-16 bg-muted animate-pulse rounded-lg" />
                ))}
              </div>
            ) : filteredPhrases.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">No phrases found</p>
            ) : (
              <div className="space-y-2 p-2">
                {filteredPhrases.map(p => (
                  <Card key={p.id}>
                    <CardContent className="flex items-center justify-between p-3">
                      <div className="min-w-0">
                        <div className="font-medium">{p.phrase}</div>
                        <div className="text-sm text-muted-foreground truncate">{p.meaning}</div>
                      </div>
                      <FamiliarityBadge level={p.familiarity} />
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </ScrollArea>
        </TabsContent>
      </Tabs>
    </div>
  )
}
