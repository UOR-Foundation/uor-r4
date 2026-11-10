interface ProjectHomeProps { projectName: string; environment: string }

export function ProjectHome({ projectName, environment }: ProjectHomeProps) {
    return <section aria-label="Project home" class="project-home">
        <h2 title={projectName}>{projectName}</h2>
        <p class="project-home-environment">Environment: <span>{environment}</span></p>
        <p class="project-home-empty">No tasks yet. Tasks appear here when this project records one.</p>
    </section>;
}
