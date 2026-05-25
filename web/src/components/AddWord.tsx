import { useState } from 'react';
import { api, endpoints } from '../api/client';
import { Plus } from 'lucide-react';

interface WordResult {
    word: string;
    translation: string;
    examples: { sentence: string; translation: string }[];
}

interface RecentItem extends WordResult {
    duplicate?: boolean;
}

interface AddWordResponse {
    word: WordResult;
    duplicate: boolean;
}

const splitBatch = (text: string): string[] => {
    const seen = new Set<string>();
    const out: string[] = [];
    for (const raw of text.split(/[,\s]+/)) {
        const token = raw.trim().toLowerCase();
        if (!token || seen.has(token)) continue;
        seen.add(token);
        out.push(token);
    }
    return out;
};

export const AddWord = () => {
    const [inputWord, setInputWord] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [batchMode, setBatchMode] = useState(false);
    const [recentWords, setRecentWords] = useState<RecentItem[]>([]);
    const [error, setError] = useState('');
    const [notice, setNotice] = useState('');

    const submitOne = async (word: string): Promise<{ item: RecentItem; duplicate: boolean } | { error: string }> => {
        try {
            const res = await api.post<AddWordResponse>(endpoints.addWord, { word, language: 'en' });
            const { word: newWord, duplicate } = res.data;
            return { item: { ...newWord, duplicate }, duplicate };
        } catch (err: unknown) {
            return { error: err instanceof Error ? err.message : 'request failed' };
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const raw = inputWord.trim();
        if (!raw) return;

        setIsLoading(true);
        setError('');
        setNotice('');

        const words = batchMode ? splitBatch(raw) : [raw];
        const added: RecentItem[] = [];
        const failed: string[] = [];
        let dupCount = 0;

        for (const w of words) {
            const result = await submitOne(w);
            if ('item' in result) {
                added.push(result.item);
                if (result.duplicate) dupCount += 1;
            } else {
                failed.push(w);
            }
        }

        if (added.length > 0) {
            setRecentWords((current) => [...added.reverse(), ...current]);
        }
        // Preserve failed words in the input so the user can retry
        setInputWord(failed.join(' '));

        if (failed.length > 0) {
            setError(`Failed to add ${failed.length} word(s): ${failed.join(', ')}`);
        }
        if (batchMode && added.length > 0) {
            setNotice(`Added ${added.length} word(s)${dupCount ? `, ${dupCount} already learned (review reset)` : ''}.`);
        } else if (!batchMode && dupCount > 0 && added[0]) {
            setNotice(`"${added[0].word}" already learned — review reset.`);
        }

        setIsLoading(false);
    };

    return (
        <div className="add-word-container">
            <div className="card">
                <h2>Add New Words</h2>
                <form onSubmit={handleSubmit} style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
                    <input
                        type="text"
                        className="input"
                        placeholder={batchMode ? "Enter multiple words separated by space/comma" : "Enter a word or phrase (e.g. 'give up')"}
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
                <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', marginTop: '0.6rem', color: '#ccc', cursor: 'pointer', userSelect: 'none' }}>
                    <input
                        type="checkbox"
                        checked={batchMode}
                        onChange={(e) => setBatchMode(e.target.checked)}
                        disabled={isLoading}
                    />
                    <span>Split into multiple words (whitespace / comma)</span>
                </label>
                {error && <p style={{ color: 'var(--error)', marginTop: '0.5rem' }}>{error}</p>}
                {notice && <p style={{ color: '#fbbf24', marginTop: '0.5rem' }}>↻ {notice}</p>}
            </div>

            <div className="recent-words">
                <h3>Recently Added</h3>
                {recentWords.length === 0 && <p style={{ color: '#666' }}>No words added yet this session.</p>}
                {recentWords.map((item, idx) => (
                    <div key={idx} className="card word-item" style={{ marginBottom: '0.5rem' }}>
                        <div>
                            <strong style={{ fontSize: '1.2em', color: item.duplicate ? '#fbbf24' : '#a5f3fc' }}>
                                {item.duplicate ? '↻ ' : ''}{item.word}
                            </strong>
                            {item.duplicate && (
                                <span style={{ marginLeft: '0.5rem', fontSize: '0.8em', color: '#fbbf24' }}>
                                    review reset
                                </span>
                            )}
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
