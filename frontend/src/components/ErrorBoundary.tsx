'use client';

import React, { ReactNode, ErrorInfo } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Global Error Boundary Component
 * Catches React component errors and displays a fallback UI
 * instead of crashing the entire application with a white screen
 */
export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details for debugging
    console.error('Error caught by ErrorBoundary:', error, errorInfo);
    
    // Update state with error details
    this.setState({
      errorInfo,
    });

    // You can also log the error to an external error reporting service here
    // Example: logErrorToService(error, errorInfo);
  }

  resetError = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: '100vh',
            backgroundColor: '#f8f9fa',
            padding: '20px',
            fontFamily:
              '-apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif',
          }}
        >
          <div
            style={{
              textAlign: 'center',
              backgroundColor: 'white',
              borderRadius: '8px',
              boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)',
              padding: '40px 30px',
              maxWidth: '600px',
              width: '100%',
            }}
          >
            <div
              style={{
                fontSize: '56px',
                marginBottom: '20px',
              }}
            >
              ⚠️
            </div>

            <h1
              style={{
                fontSize: '28px',
                fontWeight: 'bold',
                color: '#1a202c',
                marginBottom: '10px',
              }}
            >
              Something went wrong
            </h1>

            <p
              style={{
                fontSize: '16px',
                color: '#718096',
                marginBottom: '20px',
                lineHeight: '1.6',
              }}
            >
              We encountered an unexpected error while rendering this page.
              Please try again by refreshing the page or going back to home.
            </p>

            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details
                style={{
                  textAlign: 'left',
                  backgroundColor: '#f7fafc',
                  border: '1px solid #e2e8f0',
                  borderRadius: '4px',
                  padding: '15px',
                  marginBottom: '20px',
                  maxHeight: '300px',
                  overflow: 'auto',
                }}
              >
                <summary
                  style={{
                    cursor: 'pointer',
                    fontWeight: 'bold',
                    color: '#2d3748',
                    marginBottom: '10px',
                  }}
                >
                  Error Details (Development Only)
                </summary>
                <pre
                  style={{
                    fontSize: '12px',
                    color: '#742a2a',
                    backgroundColor: '#fff5f5',
                    padding: '10px',
                    borderRadius: '4px',
                    overflow: 'auto',
                    margin: '10px 0 0 0',
                  }}
                >
                  {this.state.error.toString()}
                  {'\n\n'}
                  {this.state.errorInfo?.componentStack}
                </pre>
              </details>
            )}

            <div
              style={{
                display: 'flex',
                gap: '12px',
                justifyContent: 'center',
                flexWrap: 'wrap',
              }}
            >
              <button
                onClick={this.resetError}
                style={{
                  padding: '12px 24px',
                  fontSize: '16px',
                  fontWeight: '500',
                  color: 'white',
                  backgroundColor: '#4299e1',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  transition: 'background-color 0.2s',
                }}
                onMouseEnter={(e) => {
                  (e.target as HTMLButtonElement).style.backgroundColor =
                    '#3182ce';
                }}
                onMouseLeave={(e) => {
                  (e.target as HTMLButtonElement).style.backgroundColor =
                    '#4299e1';
                }}
              >
                Try Again
              </button>

              <button
                onClick={() => (window.location.href = '/')}
                style={{
                  padding: '12px 24px',
                  fontSize: '16px',
                  fontWeight: '500',
                  color: '#4299e1',
                  backgroundColor: 'transparent',
                  border: '2px solid #4299e1',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  transition: 'background-color 0.2s',
                }}
                onMouseEnter={(e) => {
                  (e.target as HTMLButtonElement).style.backgroundColor =
                    '#ebf8ff';
                }}
                onMouseLeave={(e) => {
                  (e.target as HTMLButtonElement).style.backgroundColor =
                    'transparent';
                }}
              >
                Go Home
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
