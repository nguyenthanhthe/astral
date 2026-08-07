import { Component, type ReactNode } from 'react';
import { Button } from './Button';

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
      <div className="state-block" role="alert" style={{ minHeight: '100vh', justifyContent: 'center' }}>
        <h2 className="state-block__title">Something went wrong</h2>
        <p className="state-block__hint">{this.state.message}</p>
        <Button variant="primary" onClick={this.handleRetry}>
          Retry
        </Button>
      </div>
    );
  }
}
