import type { AgentApprovalView } from '../state/agent.js';

export interface ApprovalHost {
    id: string;
    canApprove: boolean;
}

export interface ApprovalCardProps {
    approval: AgentApprovalView;
    hostIdentity: ApprovalHost | null;
    onResolve(approvalId: string, decision: 'approved' | 'denied'): void;
}

function decisionLabel(decision: AgentApprovalView['decision']): string | null {
    if (decision === 'approved') return 'Approved by the host.';
    if (decision === 'denied') return 'Denied by the host.';
    if (decision === 'expired') return 'This approval expired before the host answered.';
    return null;
}

// The page is never the approval authority. The only evidence that can enable
// a decision is a host capability handshake carried by hostIdentity; without
// it the controls stay disabled and the exact companion guidance is shown.
export function ApprovalCard({ approval, hostIdentity, onResolve }: ApprovalCardProps) {
    const resolved = decisionLabel(approval.decision);
    const canResolve = resolved === null && hostIdentity?.canApprove === true;
    return <article class="approval-card" data-approval-id={approval.approvalId} aria-label="Approval required">
        <header class="approval-heading"><h3>Approval required</h3><span class="approval-state">{resolved === null ? 'Pending host decision' : resolved}</span></header>
        <dl class="approval-facts">
            <div><dt>Tool</dt><dd>{approval.tool}</dd></div>
            <div><dt>Scope</dt><dd>{approval.scope}</dd></div>
            <div><dt>Expiry</dt><dd><time dateTime={new Date(approval.expiresAtMs).toISOString()}>{new Date(approval.expiresAtMs).toISOString()}</time></dd></div>
        </dl>
        <p class="approval-host">Host: {hostIdentity === null ? 'not connected' : hostIdentity.id}</p>
        {!canResolve && resolved === null && <p class="approval-guidance">This action needs host approval. Approve it in the local companion.</p>}
        {resolved === null && <div class="approval-actions">
            <button type="button" disabled={!canResolve} onClick={() => onResolve(approval.approvalId, 'approved')}>Approve</button>
            <button type="button" disabled={!canResolve} onClick={() => onResolve(approval.approvalId, 'denied')}>Deny</button>
        </div>}
    </article>;
}
