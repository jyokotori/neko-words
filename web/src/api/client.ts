import axios from 'axios';

export const api = axios.create({
    baseURL: import.meta.env.VITE_API_URL || 'http://localhost:8002/api/v1',
    timeout: 10000,
});

export const endpoints = {
    addWord: '/words/',
    getDueReviews: '/reviews/due',
    submitReview: (id: string) => `/reviews/${id}/log`,
    undoReview: (id: string) => `/reviews/${id}/undo`,
};
