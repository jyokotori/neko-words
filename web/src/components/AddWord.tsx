import { useState } from 'react';
import { api, endpoints } from '../api/client';
import { Plus } from 'lucide-react';

interface WordResult {
    word: string;
    translation: string;
    examples: { sentence: string; translation: string }[];
}

export const AddWord = () => {
    const [inputWord, setInputWord] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [recentWords, setRecentWords] = useState<WordResult[]>([]);
    const [error, setError] = useState('');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!inputWord.trim()) return;

        setIsLoading(true);
        setError('');

        try {
            const res = await api.post(endpoints.addWord, { word: inputWord, language: 'en' });
            const newWord = res.data;
            setRecentWords([newWord, ...recentWords]);
            setInputWord('');
        } catch (err: any) {
            if (err.response?.status === 400) {
                setError("Word already exists (Review reset!)");
            } else {
                setError(err.message || "Failed to add word");
            }
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="add-word-container">
            <div className="card">
                <h2>Add New Words</h2>
                <form onSubmit={handleSubmit} style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
                    <input
                        type="text"
                        className="input"
                        placeholder="Enter a word (e.g. 'ephemeral')"
                        value={inputWord}
                        onChange={(e) => setInputWord(e.target.value)}
                        disabled={isLoading}
                        autoFocus
                    />
                    <button type="submit" className="btn" disabled={isLoading}>
                        {isLoading ? <span className="loading-spinner"></span> : <Plus size={20} />}
                        {isLoading ? ' Adding...' : ' Add'}
                    </button>
                </form>
                {error && <p style={{ color: 'var(--error)', marginTop: '0.5rem' }}>{error}</p>}
            </div>

            <div className="recent-words">
                <h3>Recently Added</h3>
                {recentWords.length === 0 && <p style={{ color: '#666' }}>No words added yet this session.</p>}
                {recentWords.map((item, idx) => (
                    <div key={idx} className="card word-item" style={{ marginBottom: '0.5rem' }}>
                        <div>
                            <strong style={{ fontSize: '1.2em', color: '#a5f3fc' }}>{item.word}</strong>
                            <div style={{ marginTop: '0.2rem', color: '#ccc' }}>{item.translation}</div>
                        </div>
                        <div style={{ fontSize: '0.9em', color: '#888', fontStyle: 'italic', maxWidth: '50%' }}>
                            {item.examples[0]?.sentence}
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};
