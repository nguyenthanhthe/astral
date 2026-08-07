import { Component, type ReactNode } from 'react';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  message: string;
}

/** Catches render errors so the whole window never goes blank silently. */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, message: '' };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, message: error.message };
  }

  componentDidCatch(error: Error): void {
    console.error('Astral render error:', error);
  }

  handleRetry = (): void => {
    this.setState({ hasError: false, message: '' });
  };

  render(): ReactNode {
    if (!this.state.hasError) {
      return this.props.children;
    }

    return (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '100vh',
          gap: '1rem',
          color: 'var(--text-muted)',
          fontFamily: 'Outfit, sans-serif',
          textAlign: 'center',
          padding: '2rem',
        }}
      >
        <h2 style={{ color: '#f87171', margin: 0 }}>Something went wrong</h2>
        <p style={{ margin: 0 }}>{this.state.message}</p>
        <button
          onClick={this.handleRetry}
          style={{
            padding: '0.5rem 1.2rem',
            background: '#0284c7',
            border: 'none',
            borderRadius: '6px',
            color: 'white',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          Retry
        </button>
      </div>
    );
  }
}
