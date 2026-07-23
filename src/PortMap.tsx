import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as d3 from "d3";
import "./PortMap.css";

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  port: number;
  pid: number;
  process_name: string;
  project_name: string | null;
  cluster_id: string | null;
  framework: string | null;
  is_dev: boolean;
  connection_count: number;
}

interface GraphEdge extends d3.SimulationLinkDatum<GraphNode> {
  source: string | GraphNode;
  target: string | GraphNode;
  active: boolean;
  edge_type: "tcp_connection" | "project_peer" | "orchestration_peer";
}

interface PortCluster {
  id: string;
  label: string;
  node_ids: string[];
}

interface PortGraph { nodes: GraphNode[]; edges: GraphEdge[]; clusters: PortCluster[]; }

const FW_COLORS: Record<string, string> = {
  React:"#61dafb",Vite:"#646cff",Angular:"#dd0031",
  Node:"#68a063",Django:"#2bbc8a",HTTP:"#f0a500",
  Jupyter:"#f37626",Postgres:"#336791",MySQL:"#4479a1",
  Redis:"#dc382d",Mongo:"#4db33d",PHP:"#8892bf",
  Tauri:"#ffc131",HTTPS:"#22c55e",
};

function nodeColor(n: GraphNode) {
  return (n.framework && FW_COLORS[n.framework]) || (n.is_dev ? "#7c6fff" : "#4a4a6a");
}
function nodeR(n: GraphNode) {
  return n.is_dev ? 26 : 18;
}

const EDGE_STYLE: Record<string, { dash: string; width: number; opacity: number; glow: boolean; flow: boolean }> = {
  tcp_connection:     { dash: "none",           width: 1.5, opacity: .65, glow: true,  flow: true },
  project_peer:       { dash: "6 4",            width: 1,   opacity: .35, glow: false, flow: false },
  orchestration_peer: { dash: "2 4",            width: .8,  opacity: .25, glow: false, flow: false },
};

export default function PortMap({ onClose }: { onClose: () => void }) {
  const svgRef   = useRef<SVGSVGElement>(null);
  const canvasRef= useRef<HTMLCanvasElement>(null);
  const simRef   = useRef<d3.Simulation<GraphNode,GraphEdge>|null>(null);
  const rafRef   = useRef<number>(0);
  const [graph, setGraph]       = useState<PortGraph>({nodes:[],edges:[],clusters:[]});
  const [selected, setSelected] = useState<GraphNode|null>(null);
  const [loading, setLoading]   = useState(true);

  const graphRef = useRef<string>("");

  const fetchGraph = useCallback(async () => {
    try {
      const g = await invoke<PortGraph>("get_port_graph");
      const key = JSON.stringify(g);
      if (key !== graphRef.current) {
        graphRef.current = key;
        setGraph(g);
      }
    } catch { } finally { setLoading(false); }
  }, []);

  useEffect(() => {
    fetchGraph();
    let unlisten: (() => void) | null = null;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen("ports-updated", () => { fetchGraph(); }).then((fn) => { unlisten = fn; });
    });
    return () => { if (unlisten) unlisten(); };
  }, [fetchGraph]);

  // Particle canvas background
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const cvs = canvas;
    const wrap = cvs.parentElement!;
    const ctx = cvs.getContext("2d")!;

    let W = 0, H = 0;
    let raf: number;
    let pts: Array<{x:number;y:number;vx:number;vy:number;r:number;op:number}> = [];

    function resize() {
      W = wrap.clientWidth || window.innerWidth;
      H = wrap.clientHeight || window.innerHeight;
      cvs.width = W;
      cvs.height = H;
      pts = Array.from({ length: 60 }, () => ({
        x: Math.random()*W, y: Math.random()*H,
        vx:(Math.random()-.5)*.3, vy:(Math.random()-.5)*.3,
        r: Math.random()*1.5+.5, op: Math.random()*.35+.05,
      }));
    }

    function draw() {
      if (W === 0 || H === 0) resize();
      ctx.clearRect(0,0,W,H);
      ctx.strokeStyle="rgba(124,111,255,0.04)"; ctx.lineWidth=.5;
      for(let x=0;x<W;x+=40){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,H);ctx.stroke();}
      for(let y=0;y<H;y+=40){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(W,y);ctx.stroke();}
      pts.forEach(p=>{
        p.x+=p.vx; p.y+=p.vy;
        if(p.x<0)p.x=W; if(p.x>W)p.x=0;
        if(p.y<0)p.y=H; if(p.y>H)p.y=0;
        ctx.beginPath(); ctx.arc(p.x,p.y,p.r,0,Math.PI*2);
        ctx.fillStyle=`rgba(124,111,255,${p.op})`; ctx.fill();
      });
      const vg = ctx.createRadialGradient(W/2,H/2,H*.2,W/2,H/2,H*.8);
      vg.addColorStop(0,"rgba(0,0,0,0)"); vg.addColorStop(1,"rgba(0,0,12,0.75)");
      ctx.fillStyle=vg; ctx.fillRect(0,0,W,H);
      raf = requestAnimationFrame(draw);
    }

    const observer = new ResizeObserver(() => resize());
    observer.observe(wrap);
    resize();
    draw();

    return () => { cancelAnimationFrame(raf); observer.disconnect(); };
  }, []);

  // D3 graph
  useEffect(() => {
    if (!svgRef.current || loading) return;
    const el = svgRef.current;
    const W = el.clientWidth || window.innerWidth;
    const H = el.clientHeight || window.innerHeight;
    const svg = d3.select(el);
    svg.selectAll("*").remove();
    if (simRef.current) simRef.current.stop();
    cancelAnimationFrame(rafRef.current);

    const defs = svg.append("defs");

    // Arrow marker
    defs.append("marker").attr("id","arr").attr("viewBox","0 0 10 10")
      .attr("refX",32).attr("refY",5).attr("markerWidth",6).attr("markerHeight",6)
      .attr("orient","auto-start-reverse")
      .append("path").attr("d","M2 1L8 5L2 9").attr("fill","none")
      .attr("stroke","rgba(124,111,255,0.5)").attr("stroke-width",1.5)
      .attr("stroke-linecap","round");

    // Secondary arrow for non-TCP edges
    defs.append("marker").attr("id","arr-dim").attr("viewBox","0 0 10 10")
      .attr("refX",28).attr("refY",5).attr("markerWidth",5).attr("markerHeight",5)
      .attr("orient","auto-start-reverse")
      .append("path").attr("d","M2 1L8 5L2 9").attr("fill","none")
      .attr("stroke","rgba(124,111,255,0.2)").attr("stroke-width",1)
      .attr("stroke-linecap","round");

    function glow(id: string, std: number, color: string) {
      const f = defs.append("filter").attr("id",id)
        .attr("x","-80%").attr("y","-80%").attr("width","260%").attr("height","260%");
      f.append("feFlood").attr("flood-color",color).attr("result","c");
      f.append("feComposite").attr("in","c").attr("in2","SourceAlpha").attr("operator","in").attr("result","cc");
      f.append("feGaussianBlur").attr("in","cc").attr("stdDeviation",std).attr("result","blur");
      const m = f.append("feMerge");
      m.append("feMergeNode").attr("in","blur");
      m.append("feMergeNode").attr("in","SourceGraphic");
    }
    glow("glow-sm",3,"rgba(124,111,255,0.8)");
    glow("glow-md",7,"rgba(124,111,255,0.5)");
    graph.nodes.forEach(n => glow(`gn-${n.id}`,5,nodeColor(n)));

    // Edge gradients — only for TCP edges
    const tcpEdges = graph.edges.filter(e => e.edge_type === "tcp_connection");
    tcpEdges.forEach((e, i) => {
      const s = graph.nodes.find(n=>n.id===(typeof e.source==="string"?e.source:(e.source as GraphNode).id));
      const t = graph.nodes.find(n=>n.id===(typeof e.target==="string"?e.target:(e.target as GraphNode).id));
      const sc = s ? nodeColor(s) : "#7c6fff";
      const tc = t ? nodeColor(t) : "#7c6fff";
      const g = defs.append("linearGradient").attr("id",`eg${i}`).attr("gradientUnits","userSpaceOnUse");
      g.append("stop").attr("offset","0%").attr("stop-color",sc).attr("stop-opacity",.1);
      g.append("stop").attr("offset","50%").attr("stop-color","rgba(160,150,255,0.9)");
      g.append("stop").attr("offset","100%").attr("stop-color",tc).attr("stop-opacity",.1);
    });

    const container = svg.append("g");
    svg.call(d3.zoom<SVGSVGElement,unknown>().scaleExtent([.25,4])
      .on("zoom", e => container.attr("transform", e.transform)));

    const simNodes: GraphNode[] = graph.nodes.map(n=>({...n}));
    const simEdges: GraphEdge[] = graph.edges.map(e=>({...e,
      source: typeof e.source==="string" ? e.source : (e.source as GraphNode).id,
      target: typeof e.target==="string" ? e.target : (e.target as GraphNode).id,
    }));

    // Cluster grouping force
    const clusterCenters = new Map<string, {x:number;y:number}>();
    const clusters = graph.clusters;
    if (clusters.length > 0) {
      const angleStep = (2 * Math.PI) / clusters.length;
      clusters.forEach((c, i) => {
        const angle = angleStep * i - Math.PI / 2;
        const radius = Math.min(W, H) * 0.3;
        clusterCenters.set(c.id, {
          x: W / 2 + Math.cos(angle) * radius,
          y: H / 2 + Math.sin(angle) * radius,
        });
      });
    }

    const sim = d3.forceSimulation<GraphNode>(simNodes)
      .force("link", d3.forceLink<GraphNode,GraphEdge>(simEdges)
        .id(d=>d.id).distance(140).strength(.45))
      .force("charge", d3.forceManyBody().strength(d=>(d as GraphNode).is_dev?-500:-280))
      .force("center", d3.forceCenter(W/2, H/2))
      .force("collide", d3.forceCollide<GraphNode>().radius(d=>nodeR(d)+28));

    // Cluster force: pull nodes toward their cluster center
    if (clusterCenters.size > 0) {
      sim.force("cluster", (alpha: number) => {
        for (const n of simNodes) {
          if (!n.cluster_id) continue;
          const center = clusterCenters.get(n.cluster_id);
          if (!center) continue;
          const strength = 0.08 * alpha;
          n.vx = (n.vx || 0) + (center.x - (n.x || 0)) * strength;
          n.vy = (n.vy || 0) + (center.y - (n.y || 0)) * strength;
        }
      });
    }

    simRef.current = sim;

    // Cluster background group (rendered before edges)
    const clusterG = container.append("g").attr("class","cluster-bg");

    const edgeG = container.append("g");

    // Edges split by type
    const tcpSimEdges = simEdges.filter(e => e.edge_type === "tcp_connection");
    const projSimEdges = simEdges.filter(e => e.edge_type !== "tcp_connection");

    // TCP edges: gradient, animated flow
    const tcpEdgePaths = edgeG.selectAll<SVGPathElement,GraphEdge>(".tep")
      .data(tcpSimEdges).enter().append("path")
      .attr("fill","none")
      .attr("stroke",(_,i)=>`url(#eg${i})`)
      .attr("stroke-width",1.5)
      .attr("opacity",.65)
      .attr("marker-end","url(#arr)");

    const tcpFlowPaths = edgeG.selectAll<SVGPathElement,GraphEdge>(".tfp")
      .data(tcpSimEdges).enter().append("path")
      .attr("fill","none")
      .attr("stroke","rgba(210,200,255,0.95)")
      .attr("stroke-width",2.5)
      .attr("stroke-dasharray","2 30");

    const tcpGlowPaths = edgeG.selectAll<SVGPathElement,GraphEdge>(".tgp")
      .data(tcpSimEdges).enter().append("path")
      .attr("fill","none")
      .attr("stroke","rgba(124,111,255,0.25)")
      .attr("stroke-width",6)
      .attr("filter","url(#glow-sm)");

    // Project / Orchestration edges: dashed, no flow
    const projEdgePaths = edgeG.selectAll<SVGPathElement,GraphEdge>(".pep")
      .data(projSimEdges).enter().append("path")
      .attr("fill","none")
      .attr("stroke","rgba(124,111,255,0.25)")
      .attr("stroke-width",d=>EDGE_STYLE[d.edge_type]?.width ?? 1)
      .attr("stroke-dasharray",d=>EDGE_STYLE[d.edge_type]?.dash ?? "6 4")
      .attr("opacity",d=>EDGE_STYLE[d.edge_type]?.opacity ?? .35)
      .attr("marker-end","url(#arr-dim)");

    function arcPath(s: GraphNode, t: GraphNode) {
      const dx=(t.x||0)-(s.x||0), dy=(t.y||0)-(s.y||0);
      const dr=Math.sqrt(dx*dx+dy*dy)*1.4;
      return `M${s.x},${s.y} A${dr},${dr} 0 0,1 ${t.x},${t.y}`;
    }

    // Node groups
    const nodeGs = container.append("g").selectAll<SVGGElement,GraphNode>(".ng")
      .data(simNodes).enter().append("g").attr("class","ng")
      .style("cursor","pointer")
      .call(d3.drag<SVGGElement,GraphNode>()
        .on("start",(e,d)=>{if(!e.active)sim.alphaTarget(.3).restart();d.fx=d.x;d.fy=d.y;})
        .on("drag",(e,d)=>{d.fx=e.x;d.fy=e.y;})
        .on("end",(e,d)=>{if(!e.active)sim.alphaTarget(0);d.fx=null;d.fy=null;})
      )
      .on("click",(ev,d)=>{ev.stopPropagation();setSelected(p=>p?.id===d.id?null:d);});

    svg.on("click",()=>setSelected(null));

    // Outer decorative ring
    nodeGs.append("circle")
      .attr("r",d=>nodeR(d)+26)
      .attr("fill","none")
      .attr("stroke",d=>nodeColor(d))
      .attr("stroke-width",.5)
      .attr("stroke-dasharray","2 8")
      .attr("opacity",.1);

    // Mid glow ring
    nodeGs.append("circle")
      .attr("r",d=>nodeR(d)+14)
      .attr("fill","none")
      .attr("stroke",d=>nodeColor(d))
      .attr("stroke-width",.8)
      .attr("opacity",.18)
      .attr("filter",d=>`url(#gn-${d.id})`);

    // Core
    nodeGs.append("circle").attr("class","core")
      .attr("r",d=>nodeR(d))
      .attr("fill",d=>`${nodeColor(d)}1a`)
      .attr("stroke",d=>nodeColor(d))
      .attr("stroke-width",1.5)
      .attr("filter",d=>`url(#gn-${d.id})`);

    // Inner bright spot
    nodeGs.append("circle")
      .attr("r",d=>nodeR(d)*.3)
      .attr("fill",d=>nodeColor(d))
      .attr("opacity",.45)
      .attr("filter",d=>`url(#gn-${d.id})`);

    // Port label
    nodeGs.append("text")
      .attr("text-anchor","middle").attr("dominant-baseline","central")
      .attr("dy",d=>d.project_name?"-9":"0")
      .attr("font-family","JetBrains Mono,monospace")
      .attr("font-size",d=>d.is_dev?"12":"10")
      .attr("font-weight","700")
      .attr("fill",d=>nodeColor(d))
      .attr("letter-spacing","-.01em")
      .text(d=>`:${d.port}`);

    // Project name
    nodeGs.filter(d=>!!d.project_name).append("text")
      .attr("text-anchor","middle").attr("dominant-baseline","central")
      .attr("dy","8").attr("font-family","Geist,sans-serif").attr("font-size","8")
      .attr("fill","rgba(255,255,255,0.35)")
      .text(d=>d.project_name!.slice(0,10));

    // Framework badge
    nodeGs.filter(d=>!!d.framework).append("text")
      .attr("text-anchor","middle")
      .attr("font-family","Geist,sans-serif").attr("font-size","7")
      .attr("font-weight","700").attr("letter-spacing",".1em")
      .attr("fill",d=>nodeColor(d)).attr("opacity",.75)
      .attr("dy",d=>d.is_dev?-42:-32)
      .text(d=>d.framework!.toUpperCase());

    // Connection count badge
    nodeGs.filter(d=>d.connection_count>0).append("circle")
      .attr("cx",d=>nodeR(d)-2).attr("cy",d=>-(nodeR(d)-2))
      .attr("r",7).attr("fill","#080818")
      .attr("stroke","rgba(124,111,255,0.45)").attr("stroke-width",1);

    nodeGs.filter(d=>d.connection_count>0).append("text")
      .attr("x",d=>nodeR(d)-2).attr("y",d=>-(nodeR(d)-2))
      .attr("text-anchor","middle").attr("dominant-baseline","central")
      .attr("font-family","JetBrains Mono,monospace")
      .attr("font-size","7").attr("font-weight","700")
      .attr("fill","rgba(200,190,255,0.9)")
      .text(d=>String(d.connection_count));

    // Tick
    sim.on("tick",()=>{
      const bounds = computeClusterBounds(simNodes);
      renderClusterBounds(clusterG, bounds, clusters);

      const path=(d: GraphEdge)=>arcPath(d.source as GraphNode, d.target as GraphNode);
      tcpEdgePaths.attr("d",path);
      tcpFlowPaths.attr("d",path);
      tcpGlowPaths.attr("d",path);
      projEdgePaths.attr("d",path);
      nodeGs.attr("transform",d=>`translate(${d.x||0},${d.y||0})`);
    });

    // Animate flow on TCP edges only
    let off=0;
    function anim(){
      off-=.9;
      tcpFlowPaths.attr("stroke-dashoffset",off);
      tcpGlowPaths.attr("stroke-dashoffset",off*.5);
      rafRef.current=requestAnimationFrame(anim);
    }
    anim();

    // Pulse dev nodes
    function pulseNodes(){
      nodeGs.filter(d=>d.is_dev).select(".core")
        .transition().duration(1400).attr("opacity",.9)
        .transition().duration(1400).attr("opacity",.6)
        .on("end",pulseNodes);
    }
    setTimeout(pulseNodes,300);

    // Entrance
    nodeGs.attr("opacity",0)
      .transition().delay((_,i)=>i*70).duration(500).attr("opacity",1);

    return () => { sim.stop(); cancelAnimationFrame(rafRef.current); };
  }, [graph, loading]);

  // Highlight selected
  useEffect(()=>{
    if(!svgRef.current)return;
    const svg=d3.select(svgRef.current);
    svg.selectAll<SVGCircleElement,GraphNode>(".core")
      .attr("stroke-width",d=>selected?.id===d.id?3:1.5)
      .attr("fill",d=>selected?.id===d.id?`${nodeColor(d)}35`:`${nodeColor(d)}1a`);
  },[selected]);

  const clusterCount = graph.clusters.length;

  return (
    <div className="pm-wrap">
      <canvas ref={canvasRef} className="pm-bg" />
      <svg ref={svgRef} className="pm-svg" />

      <div className="pm-header">
        <div className="pm-header-left">
          <div className="pm-dot" />
          <div>
            <div className="pm-title">PORT MAP</div>
            <div className="pm-sub">
              {graph.nodes.length} nodes · {graph.edges.length} connections
              {clusterCount > 0 && ` · ${clusterCount} clusters`}
            </div>
          </div>
        </div>
        <div className="pm-header-right">
          <span className="pm-scan-txt">LIVE SCAN</span>
          <div className="pm-pulse" />
          <button className="pm-icon-btn pm-refresh-btn" onClick={fetchGraph}>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M10 6A4 4 0 1 1 6 2M6 2l2-2M6 2L4 0"
                stroke="currentColor" strokeWidth="1.5"
                strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
          <button className="pm-icon-btn pm-close-btn" onClick={onClose}>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M1 1l10 10M11 1L1 11"
                stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
            </svg>
          </button>
        </div>
      </div>

      {loading && (
        <div className="pm-loading">
          <div className="pm-loader" />
          <span>Mapping connections…</span>
        </div>
      )}

      {!loading && graph.nodes.length===0 && (
        <div className="pm-empty">
          <div className="pm-empty-ring" />
          <span>No ports to map</span>
          <span className="pm-empty-sub">Start some servers and come back</span>
        </div>
      )}

      {selected && (
        <div className="pm-inspector">
          <div className="pmi-port" style={{color:nodeColor(selected)}}>
            :{selected.port}
          </div>
          <div className="pmi-name">
            {selected.project_name ?? selected.process_name}
          </div>
          {selected.framework && (
            <span className="pmi-badge" style={{
              color:nodeColor(selected),
              borderColor:`${nodeColor(selected)}55`,
              background:`${nodeColor(selected)}18`,
            }}>
              {selected.framework}
            </span>
          )}
          {selected.cluster_id && (
            <div className="pmi-row"><span>Cluster</span><span>{selected.cluster_id}</span></div>
          )}
          <div className="pmi-row"><span>PID</span><span>{selected.pid}</span></div>
          <div className="pmi-row"><span>Process</span><span>{selected.process_name}</span></div>
          <div className="pmi-row"><span>Connections</span><span>{selected.connection_count}</span></div>
          <button className="pmi-kill" onClick={async()=>{
            await invoke("kill_process",{pid:selected.pid});
            setSelected(null); fetchGraph();
          }}>
            Kill process
          </button>
        </div>
      )}

      <div className="pm-legend">
        <div className="pm-leg-items">
          {[["#61dafb","Dev server"],["#4a4a6a","System"],["#22c55e","Database"]]
            .map(([c,l])=>(
              <div key={l} className="pm-leg-item">
                <div className="pm-leg-dot" style={{background:c,boxShadow:`0 0 4px ${c}`}}/>
                {l}
              </div>
            ))}
          <div className="pm-leg-sep" />
          <div className="pm-leg-item">
            <svg width="14" height="4" viewBox="0 0 14 4"><line x1="0" y1="2" x2="14" y2="2" stroke="rgba(124,111,255,0.5)" strokeWidth="1.5"/></svg>
            TCP
          </div>
          <div className="pm-leg-item">
            <svg width="14" height="4" viewBox="0 0 14 4"><line x1="0" y1="2" x2="14" y2="2" stroke="rgba(124,111,255,0.25)" strokeWidth="1" strokeDasharray="4 4"/></svg>
            Project
          </div>
        </div>
        <span className="pm-leg-txt">DRAG · SCROLL TO ZOOM · CLICK TO INSPECT</span>
      </div>
    </div>
  );
}

function computeClusterBounds(nodes: GraphNode[]): Map<string, {x:number;y:number;w:number;h:number}> {
  const groups = new Map<string, {xs:number[];ys:number[]}>();
  for (const n of nodes) {
    if (!n.cluster_id) continue;
    if (!groups.has(n.cluster_id)) groups.set(n.cluster_id, {xs:[],ys:[]});
    const g = groups.get(n.cluster_id)!;
    g.xs.push(n.x || 0);
    g.ys.push(n.y || 0);
  }
  const pad = 60;
  const result = new Map<string, {x:number;y:number;w:number;h:number}>();
  for (const [id, g] of groups) {
    const minX = Math.min(...g.xs);
    const maxX = Math.max(...g.xs);
    const minY = Math.min(...g.ys);
    const maxY = Math.max(...g.ys);
    result.set(id, {
      x: minX - pad,
      y: minY - pad - 14,
      w: maxX - minX + pad * 2,
      h: maxY - minY + pad * 2 + 14,
    });
  }
  return result;
}

function renderClusterBounds(g: d3.Selection<SVGGElement, unknown, null, undefined>, bounds: Map<string, {x:number;y:number;w:number;h:number}>, clusters: PortCluster[]) {
  g.selectAll(".cluster-box").remove();
  for (const c of clusters) {
    const b = bounds.get(c.id);
    if (!b) continue;
    const grp = g.append("g").attr("class","cluster-box");
    grp.append("rect")
      .attr("x",b.x).attr("y",b.y)
      .attr("width",b.w).attr("height",b.h)
      .attr("rx",12).attr("ry",12)
      .attr("fill","rgba(124,111,255,0.03)")
      .attr("stroke","rgba(124,111,255,0.12)")
      .attr("stroke-width",1)
      .attr("stroke-dasharray","4 4");
    grp.append("text")
      .attr("x",b.x + 10).attr("y",b.y + 14)
      .attr("font-family","system-ui")
      .attr("font-size","9")
      .attr("font-weight","600")
      .attr("fill","rgba(124,111,255,0.4)")
      .attr("letter-spacing","0.05em")
      .text(c.label);
  }
}
