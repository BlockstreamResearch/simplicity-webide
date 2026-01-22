// DAG Viewer using D3.js for Simplicity program visualization

let dagSvg, dagZoom, dagZoomInitialTransform;
let dagRenderedFor = null;  // Track which JSON we've rendered to avoid re-renders

// Color scheme for different node types
const kindColors = {
    jet: '#ff9517',      // Orange - jets are important
    leaf: '#4ade80',     // Green - leaf nodes (witness, word, fail)
    plumbing: '#6b7280', // Gray - structural nodes (iden, unit, comp, take, drop)
    sum: '#8b5cf6',      // Purple - sum types (injl, injr, case)
    product: '#3b82f6',  // Blue - product types (pair)
    assertion: '#ef4444', // Red - assertions
    advanced: '#f59e0b', // Amber - advanced (disconnect)
};

// Make renderDag available globally for Rust to call
window.renderDag = function(containerId, dagJson) {
    const container = document.getElementById(containerId);
    if (!container) return;

    if (dagRenderedFor === dagJson && container.querySelector('svg')) {
        return;
    }
    dagRenderedFor = dagJson;

    let dag;
    try {
        dag = JSON.parse(dagJson);
    } catch (e) {
        console.error('Failed to parse DAG JSON:', e);
        return;
    }

    if (!dag.nodes || dag.nodes.length === 0) {
        container.innerHTML = '<div class="dag-empty">No nodes to display</div>';
        return;
    }

    // Check node count limit
    if (dag.nodes.length > 500) {
        container.innerHTML = `<div class="dag-error">Too many nodes to display (${dag.nodes.length}). Consider simplifying the program.</div>`;
        return;
    }

    container.innerHTML = '';

    // Create SVG
    const width = container.clientWidth || 800;
    const height = container.clientHeight || 500;

    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    container.appendChild(svgEl);

    dagSvg = d3.select(svgEl)
        .attr('width', width)
        .attr('height', height);

    // Build hierarchy from DAG
    const nodeMap = new Map();
    dag.nodes.forEach(n => nodeMap.set(n.id, { ...n, children: [] }));

    // Build parent-child relationships
    dag.edges.forEach(e => {
        const parent = nodeMap.get(e.from);
        const child = nodeMap.get(e.to);
        if (parent && child) {
            parent.children.push(child);
        }
    });

    // Find root node
    const root = nodeMap.get(dag.root_id) || nodeMap.values().next().value;
    if (!root) {
        container.innerHTML = '<div class="dag-error">No root node found</div>';
        return;
    }

    // Create D3 hierarchy
    const hierarchy = d3.hierarchy(root, d => d.children);

    // Layout settings
    const nodeSize = [140, 36];
    const nodeGap = [50, 20];

    const treeLayout = d3.tree()
        .nodeSize([nodeSize[1] + nodeGap[1], nodeSize[0] + nodeGap[0]]);

    const treeData = treeLayout(hierarchy);

    // Set up zoom
    const zoomG = dagSvg.append('g');

    dagZoom = d3.zoom()
        .scaleExtent([0.1, 3])
        .on('zoom', (e) => {
            zoomG.attr('transform', e.transform);
        });

    dagSvg.call(dagZoom);

    // Calculate tree bounds for fit-to-view
    const nodes = treeData.descendants();
    const xExtent = d3.extent(nodes, d => d.x);
    const yExtent = d3.extent(nodes, d => d.y);

    const treeBounds = {
        minX: xExtent[0] - nodeSize[1] / 2,
        maxX: xExtent[1] + nodeSize[1] / 2,
        minY: yExtent[0] - nodeSize[0] / 2,
        maxY: yExtent[1] + nodeSize[0] / 2
    };

    const treeWidth = treeBounds.maxY - treeBounds.minY;
    const treeHeight = treeBounds.maxX - treeBounds.minX;

    // Calculate scale to fit with padding
    const padding = 40;
    const scaleX = (width - padding * 2) / treeWidth;
    const scaleY = (height - padding * 2) / treeHeight;
    const scale = Math.min(scaleX, scaleY, 1);  // Don't zoom in past 1x

    // Calculate center offset to fit the tree in viewport
    const centerX = width / 2 - (treeBounds.minY + treeWidth / 2) * scale;
    const centerY = height / 2 - (treeBounds.minX + treeHeight / 2) * scale;

    dagZoomInitialTransform = d3.zoomIdentity.translate(centerX, centerY).scale(scale);
    dagSvg.call(dagZoom.transform, dagZoomInitialTransform);

    const graphG = zoomG.append('g');

    // Draw edges
    const links = treeData.links();
    graphG.selectAll('path.dag-edge')
        .data(links)
        .enter()
        .append('path')
        .attr('class', 'dag-edge')
        .attr('d', d => {
            const midY = (d.source.y + d.target.y) / 2;
            return `M${d.source.y},${d.source.x} C${midY},${d.source.x} ${midY},${d.target.x} ${d.target.y},${d.target.x}`;
        });

    // Draw nodes
    const nodeGroups = graphG.selectAll('g.dag-node')
        .data(treeData.descendants())
        .enter()
        .append('g')
        .attr('class', 'dag-node')
        .attr('transform', d => `translate(${d.y}, ${d.x})`)
        .style('cursor', 'pointer')
        .on('click', (event, d) => {
            // Call the Rust callback via window
            if (window.dagNodeClickHandler) {
                window.dagNodeClickHandler(d.data.id);
            }
            // Highlight selected node
            graphG.selectAll('.dag-node-rect').classed('selected', false);
            d3.select(event.currentTarget).select('.dag-node-rect').classed('selected', true);
        });

    // Node rectangles
    nodeGroups.append('rect')
        .attr('class', 'dag-node-rect')
        .attr('x', -nodeSize[0] / 2)
        .attr('y', -nodeSize[1] / 2)
        .attr('width', nodeSize[0])
        .attr('height', nodeSize[1])
        .attr('rx', 6)
        .attr('ry', 6)
        .style('fill', d => kindColors[d.data.kind_class] || '#6b7280')
        .style('stroke', d => d3.color(kindColors[d.data.kind_class] || '#6b7280').darker(0.5))
        .style('stroke-width', 2);

    // Node labels
    nodeGroups.append('text')
        .attr('class', 'dag-node-label')
        .attr('text-anchor', 'middle')
        .attr('dominant-baseline', 'middle')
        .text(d => {
            const label = d.data.kind;
            return label.length > 12 ? label.slice(0, 10) + '..' : label;
        });

    // Set up zoom controls
    setupZoomControls();
}

function setupZoomControls() {
    const zoomInBtn = document.getElementById('dag-zoom-in');
    const zoomOutBtn = document.getElementById('dag-zoom-out');
    const zoomResetBtn = document.getElementById('dag-zoom-reset');

    if (zoomInBtn) {
        zoomInBtn.onclick = () => dagSvg?.transition().call(dagZoom.scaleBy, 1.3);
    }
    if (zoomOutBtn) {
        zoomOutBtn.onclick = () => dagSvg?.transition().call(dagZoom.scaleBy, 0.7);
    }
    if (zoomResetBtn) {
        zoomResetBtn.onclick = () => dagSvg?.transition().call(dagZoom.transform, dagZoomInitialTransform);
    }
}

// Manual zoom function (exposed for external control)
window.manualDagZoom = function(mode) {
    if (!dagSvg || !dagZoom) return;
    
    if (mode === 'zoom_in') {
        dagSvg.transition().call(dagZoom.scaleBy, 1.3);
    } else if (mode === 'zoom_out') {
        dagSvg.transition().call(dagZoom.scaleBy, 0.7);
    } else if (mode === 'zoom_reset') {
        dagSvg.transition().call(dagZoom.transform, dagZoomInitialTransform);
    }
};

