export type TaskErrorScope = 'run' | 'action';

export interface TaskErrorProps {
    code: string;
    message: string;
    retryable?: boolean;
    scope?: TaskErrorScope;
    onRetry?(): void;
}

export function TaskError({ code, message, retryable = false, scope = 'run', onRetry }: TaskErrorProps) {
    return <section class="task-error" data-task-error={scope} role="alert">
        <p class="task-error-code">{code}</p>
        <p class="task-error-message">{message}</p>
        {retryable && onRetry && <button type="button" class="task-error-retry" onClick={onRetry}>Retry</button>}
    </section>;
}
