import { useQuery } from '@tanstack/react-query'
import { listWords, getWord } from './api'

export const useWords = () =>
  useQuery({ queryKey: ['words'], queryFn: listWords })

export const useWord = (word: string) =>
  useQuery({ queryKey: ['word', word], queryFn: () => getWord(word), enabled: !!word })
