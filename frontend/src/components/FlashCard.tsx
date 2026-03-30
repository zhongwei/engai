import { useState, type ReactNode } from 'react';

interface FlashCardProps {
  front: ReactNode;
  back: ReactNode;
  className?: string;
}

export default function FlashCard({ front, back, className = '' }: FlashCardProps) {
  const [flipped, setFlipped] = useState(false);

  return (
    <div
      className={`cursor-pointer perspective-1000 ${className}`}
      onClick={() => setFlipped(!flipped)}
    >
      <div
        className={`transition-transform duration-300 ${flipped ? '[transform:rotateY(180deg)]' : ''}`}
        style={{ transformStyle: 'preserve-3d' }}
      >
        <div className={flipped ? 'hidden' : ''}>{front}</div>
        <div className={flipped ? '' : 'hidden'}>{back}</div>
      </div>
    </div>
  );
}
