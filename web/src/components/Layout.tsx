import { Link } from '@tanstack/react-router'
import { LayoutDashboard, BookOpen, RefreshCw, FileText, MessageSquare } from 'lucide-react'

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/vocabulary', icon: BookOpen, label: 'Vocabulary' },
  { to: '/review', icon: RefreshCw, label: 'Review' },
  { to: '/readings', icon: FileText, label: 'Reading' },
  { to: '/chat', icon: MessageSquare, label: 'Chat' },
]

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen bg-background">
      <aside className="w-56 bg-slate-900 text-white flex flex-col shrink-0">
        <div className="p-4 text-lg font-bold border-b border-slate-700">
          Engai
        </div>
        <nav className="flex-1 p-2 space-y-1">
          {navItems.map(({ to, icon: Icon, label }) => (
            <Link
              key={to}
              to={to}
              activeProps={{
                className: 'bg-slate-700 text-white',
              }}
              className="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors text-slate-300 hover:bg-slate-800 hover:text-white"
            >
              <Icon size={18} />
              {label}
            </Link>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-auto">
        {children}
      </main>
    </div>
  )
}
