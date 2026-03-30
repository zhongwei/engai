import { Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Vocabulary from './pages/Vocabulary'
import WordCard from './pages/WordCard'
import Review from './pages/Review'
import Reading from './pages/Reading'
import Chat from './pages/Chat'

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/vocabulary" element={<Vocabulary />} />
        <Route path="/words/:word" element={<WordCard />} />
        <Route path="/review" element={<Review />} />
        <Route path="/readings" element={<Reading />} />
        <Route path="/chat" element={<Chat />} />
      </Route>
    </Routes>
  )
}
