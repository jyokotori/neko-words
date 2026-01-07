import { useState, useEffect, useCallback } from 'react';
import { api, endpoints } from '../api/client';
import { Eye, CheckCircle, Undo2, Volume2 } from 'lucide-react';

// Youdao dictionary pronunciation API (US English)
const playPronunciation = (word: string) => {
    const audio = new Audio(`https://dict.youdao.com/dictvoice?type=0&audio=${encodeURIComponent(word)}`);
    audio.play().catch(err => console.error('Audio playback failed:', err));
};

interface ReviewItem {
    word: {
        id: string;
        word: string;
        translation: string;
        examples: { sentence: string; translation: string }[];
    };
    review: any;
}

export const Review = () => {
    const [items, setItems] = useState<ReviewItem[]>([]);
    const [currentIndex, setCurrentIndex] = useState(0);
    const [isRevealed, setIsRevealed] = useState(false);
    const [loading, setLoading] = useState(true);
    const [completed, setCompleted] = useState(false);
    const [lastReviewedWordId, setLastReviewedWordId] = useState<string | null>(null);
    const [undoing, setUndoing] = useState(false);
    const [hasInteracted, setHasInteracted] = useState(false);

    useEffect(() => {
        fetchReviews();
    }, []);

    // Auto-play pronunciation when word changes (only after user interaction)
    useEffect(() => {
        if (hasInteracted && currentItem?.word.word && !loading && !completed) {
            playPronunciation(currentItem.word.word);
        }
    }, [currentIndex, items, loading, completed, hasInteracted]);

    const fetchReviews = async () => {
        setLoading(true);
        try {
            const res = await api.get(endpoints.getDueReviews, { params: { limit: 20 } });
            setItems(res.data);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const currentItem = items[currentIndex];

    const handleGrade = async (grade: string) => {
        if (!currentItem) return;
        setHasInteracted(true);

        try {
            await api.post(endpoints.submitReview(currentItem.word.id), { grade });
            setLastReviewedWordId(currentItem.word.id);

            if (currentIndex < items.length - 1) {
                setCurrentIndex(currentIndex + 1);
                setIsRevealed(false);
            } else {
                setCompleted(true);
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleUndo = useCallback(async () => {
        if (!lastReviewedWordId || undoing) return;

        setUndoing(true);
        try {
            await api.post(endpoints.undoReview(lastReviewedWordId));

            // If we completed, go back to the last card
            if (completed) {
                setCompleted(false);
                setIsRevealed(false);
            } else if (currentIndex > 0) {
                // Go back to previous card
                setCurrentIndex(currentIndex - 1);
                setIsRevealed(false);
            }

            setLastReviewedWordId(null);
        } catch (err) {
            console.error('Undo failed:', err);
        } finally {
            setUndoing(false);
        }
    }, [lastReviewedWordId, undoing, completed, currentIndex]);

    // Keyboard shortcut: Ctrl+Z / Cmd+Z
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
                e.preventDefault();
                handleUndo();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [handleUndo]);

    if (loading) return <div style={{ textAlign: 'center', marginTop: '4rem' }}>Loading reviews...</div>;

    if (completed) {
        return (
            <div className="card" style={{ textAlign: 'center', maxWidth: 400, margin: '4rem auto' }}>
                <CheckCircle size={64} color="var(--success)" style={{ margin: '0 auto 1rem' }} />
                <h2>All Done!</h2>
                <p>You have reviewed all due cards.</p>
                <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'center', marginTop: '1rem' }}>
                    {lastReviewedWordId && (
                        <button className="btn" onClick={handleUndo} disabled={undoing} style={{ opacity: undoing ? 0.6 : 1 }}>
                            <Undo2 size={18} style={{ marginRight: 6, verticalAlign: 'middle' }} />
                            Undo
                        </button>
                    )}
                    <button className="btn" onClick={() => window.location.reload()}>
                        Refresh
                    </button>
                </div>
            </div>
        );
    }

    if (items.length === 0) {
        return (
            <div className="card" style={{ textAlign: 'center', maxWidth: 400, margin: '4rem auto' }}>
                <h2>No Reviews Due</h2>
                <p>Go add some more words!</p>
            </div>
        );
    }

    return (
        <div className="review-container">
            <div className="card review-card">
                <div className="word-display">
                    {currentItem.word.word}
                    <button
                        onClick={() => {
                            setHasInteracted(true);
                            playPronunciation(currentItem.word.word);
                        }}
                        style={{
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            marginLeft: '0.5rem',
                            padding: '4px',
                            color: '#888',
                            verticalAlign: 'middle',
                        }}
                        title="Play pronunciation (US)"
                    >
                        <Volume2 size={24} />
                    </button>
                </div>

                {!isRevealed && (
                    <div style={{ marginBottom: '1rem', fontStyle: 'italic', color: '#888' }}>
                        "{currentItem.word.examples[0]?.sentence}"
                    </div>
                )}

                {isRevealed && (
                    <div className="translation-display visible">
                        <div>{currentItem.word.translation}</div>
                        <div style={{ marginTop: '1rem', textAlign: 'left', fontSize: '1rem', color: '#ccc' }}>
                            {currentItem.word.examples.map((ex, i) => (
                                <div key={i} style={{ marginBottom: '0.5rem' }}>
                                    <div>• {ex.sentence}</div>
                                    <div style={{ fontSize: '0.9em', color: '#888' }}>{ex.translation}</div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {!isRevealed ? (
                    <button className="btn" onClick={() => setIsRevealed(true)} style={{ marginTop: '2rem' }}>
                        <Eye size={18} style={{ marginRight: 8, verticalAlign: 'middle' }} /> Reveal Answer
                    </button>
                ) : (
                    <div className="grade-buttons">
                        <button className="grade-btn again" onClick={() => handleGrade('again')}>Again</button>
                        <button className="grade-btn hard" onClick={() => handleGrade('hard')}>Hard</button>
                        <button className="grade-btn good" onClick={() => handleGrade('good')}>Good</button>
                        <button className="grade-btn easy" onClick={() => handleGrade('easy')}>Easy</button>
                    </div>
                )}

                <div style={{ position: 'absolute', top: 10, right: 10, fontSize: '0.8em', color: '#666' }}>
                    Card {currentIndex + 1} / {items.length}
                </div>

                {/* Undo button - mobile friendly */}
                {lastReviewedWordId && (
                    <button
                        onClick={handleUndo}
                        disabled={undoing}
                        style={{
                            position: 'absolute',
                            top: 10,
                            left: 10,
                            background: 'rgba(255,255,255,0.1)',
                            border: '1px solid rgba(255,255,255,0.2)',
                            borderRadius: 8,
                            padding: '6px 12px',
                            color: '#aaa',
                            cursor: 'pointer',
                            display: 'flex',
                            alignItems: 'center',
                            gap: 4,
                            fontSize: '0.8em',
                            opacity: undoing ? 0.5 : 1,
                        }}
                    >
                        <Undo2 size={14} /> Undo
                    </button>
                )}
            </div>
        </div>
    );
};
