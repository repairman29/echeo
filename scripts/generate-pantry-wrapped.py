#!/usr/bin/env python3
"""
Generate Pantry Wrapped - Your 2025 Developer Year in Review
Spotify Wrapped-style narrative celebrating your accomplishments
"""

import json
import os
from pathlib import Path
from reportlab.lib import colors
from reportlab.lib.pagesizes import letter
from reportlab.lib.units import inch
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak
from reportlab.pdfgen import canvas
from reportlab.graphics.shapes import Drawing, Rect
from reportlab.graphics.charts.piecharts import Pie
from reportlab.graphics.charts.barcharts import VerticalBarChart
from reportlab.graphics import renderPDF
import math
from datetime import datetime

# Vibrant color palette
COLORS = {
    'spotify_green': colors.HexColor('#1DB954'),
    'dark_bg': colors.HexColor('#121212'),
    'card_bg': colors.HexColor('#1E1E1E'),
    'accent_purple': colors.HexColor('#8B5CF6'),
    'accent_cyan': colors.HexColor('#0EA5E9'),
    'accent_amber': colors.HexColor('#F59E0B'),
    'text_primary': colors.HexColor('#FFFFFF'),
    'text_secondary': colors.HexColor('#B3B3B3'),
    'highlight': colors.HexColor('#1DB954'),
}

def load_catalog_data():
    """Load the catalog JSON data"""
    script_dir = Path(__file__).parent
    catalog_path = script_dir.parent / 'docs' / 'repo-catalog' / 'COMPLETE_CATALOG_FIXED.json'
    
    with open(catalog_path, 'r') as f:
        data = json.load(f)
    
    return data

def format_number(num):
    """Format large numbers with commas"""
    return f"{num:,}"

def calculate_achievements(data):
    """Calculate fun achievements and stats"""
    repos = data['repos']
    summary = data['summary']
    
    # Get top repos
    top_repos = sorted([r for r in repos if r.get('capabilities', 0) > 0], 
                      key=lambda x: x.get('capabilities', 0), reverse=True)
    
    achievements = {
        'total_capabilities': summary['totalCapabilities'],
        'total_repos': summary['totalRepos'],
        'total_files': summary['totalFiles'],
        'total_signals': summary['totalSignals'],
        'top_repo': top_repos[0] if top_repos else None,
        'top_5_total': sum(r.get('capabilities', 0) for r in top_repos[:5]),
        'private_repos': summary['privateRepos'],
        'public_repos': summary['publicRepos'],
        'mega_repos': sum(1 for r in repos if r.get('capabilities', 0) >= 1000),
        'avg_per_repo': float(summary['averageCapabilities']),
        'top_10_repos': top_repos[:10],
    }
    
    # Calculate fun comparisons
    achievements['lines_estimate'] = summary['totalFiles'] * 150  # Rough estimate
    achievements['developer_percentile'] = 99  # Top 1% based on scale
    achievements['product_potential'] = summary['totalCapabilities'] // 10
    
    return achievements

def set_dark_background(canvas_obj, doc):
    """Set dark background for all pages"""
    canvas_obj.saveState()
    width, height = letter
    canvas_obj.setFillColor(COLORS['dark_bg'])
    canvas_obj.rect(0, 0, width, height, fill=1)
    canvas_obj.restoreState()

def create_wrapped_title_page(canvas_obj, doc):
    """Create Spotify Wrapped-style title page"""
    canvas_obj.saveState()
    
    width, height = letter
    
    # Dark background
    canvas_obj.setFillColor(COLORS['dark_bg'])
    canvas_obj.rect(0, 0, width, height, fill=1)
    
    # Title
    canvas_obj.setFillColor(COLORS['text_primary'])
    canvas_obj.setFont("Helvetica-Bold", 64)
    canvas_obj.drawCentredString(width/2, height - 150, "YOUR")
    
    canvas_obj.setFillColor(COLORS['highlight'])
    canvas_obj.setFont("Helvetica-Bold", 64)
    canvas_obj.drawCentredString(width/2, height - 220, "PANTRY")
    
    canvas_obj.setFillColor(COLORS['text_secondary'])
    canvas_obj.setFont("Helvetica", 28)
    canvas_obj.drawCentredString(width/2, height - 280, "2025 Developer Year in Review")
    
    # Main stat
    canvas_obj.setFillColor(COLORS['highlight'])
    canvas_obj.setFont("Helvetica-Bold", 96)
    canvas_obj.drawCentredString(width/2, height/2 + 40, "20,946")
    
    canvas_obj.setFillColor(COLORS['text_primary'])
    canvas_obj.setFont("Helvetica", 32)
    canvas_obj.drawCentredString(width/2, height/2 - 30, "Capabilities Built")
    
    # Footer
    canvas_obj.setFont("Helvetica", 14)
    canvas_obj.setFillColor(COLORS['text_secondary'])
    canvas_obj.drawCentredString(width/2, 80, "Ready to see what you've built?")
    canvas_obj.drawCentredString(width/2, 60, "Let's dive in...")
    
    canvas_obj.restoreState()

def create_achievement_page(story, achievements):
    """Create page celebrating achievements"""
    styles = getSampleStyleSheet()
    
    # Dark background style
    title_style = ParagraphStyle(
        'WrappedTitle',
        parent=styles['Title'],
        fontSize=42,
        textColor=COLORS['highlight'],
        spaceAfter=30,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    story.append(Spacer(1, 0.5*inch))
    story.append(Paragraph("2025", title_style))
    
    subtitle_style = ParagraphStyle(
        'WrappedSubtitle',
        parent=styles['Normal'],
        fontSize=24,
        textColor=COLORS['text_primary'],
        spaceAfter=40,
        alignment=TA_CENTER,
        fontName='Helvetica'
    )
    story.append(Paragraph("You've been busy...", subtitle_style))
    story.append(Spacer(1, 0.3*inch))
    
    # Achievement cards
    achievements_list = [
        (f"{format_number(achievements['total_capabilities'])}", "Capabilities Built", 
         "That's enough code to build 2,000+ products!"),
        (f"{format_number(achievements['total_repos'])}", "Repositories", 
         f"{achievements['private_repos']} private, {achievements['public_repos']} public"),
        (f"{format_number(achievements['total_files'])}", "Files Written", 
         f"Roughly {format_number(achievements['lines_estimate'])} lines of code!"),
        (f"{format_number(achievements['total_signals'])}", "Signals Detected", 
         "Your code is full of potential!"),
    ]
    
    for num, label, desc in achievements_list:
        card_style = ParagraphStyle(
            'AchievementCard',
            parent=styles['Normal'],
            fontSize=36,
            textColor=COLORS['highlight'],
            spaceAfter=5,
            alignment=TA_CENTER,
            fontName='Helvetica-Bold'
        )
        story.append(Paragraph(num, card_style))
        
        label_style = ParagraphStyle(
            'AchievementLabel',
            parent=styles['Normal'],
            fontSize=18,
            textColor=COLORS['text_primary'],
            spaceAfter=10,
            alignment=TA_CENTER,
            fontName='Helvetica-Bold'
        )
        story.append(Paragraph(label, label_style))
        
        desc_style = ParagraphStyle(
            'AchievementDesc',
            parent=styles['Normal'],
            fontSize=12,
            textColor=COLORS['text_secondary'],
            spaceAfter=30,
            alignment=TA_CENTER,
            fontName='Helvetica'
        )
        story.append(Paragraph(desc, desc_style))

def create_top_repo_page(story, achievements):
    """Create page celebrating top repository"""
    styles = getSampleStyleSheet()
    
    top_repo = achievements['top_repo']
    if not top_repo:
        return
    
    title_style = ParagraphStyle(
        'WrappedTitle',
        parent=styles['Title'],
        fontSize=36,
        textColor=COLORS['highlight'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    story.append(Spacer(1, 0.3*inch))
    story.append(Paragraph("Your Top Repository", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Repo name
    repo_name = top_repo.get('full_name', 'Unknown')
    if top_repo.get('private'):
        repo_name += ' 🔒'
    
    repo_style = ParagraphStyle(
        'TopRepo',
        parent=styles['Normal'],
        fontSize=32,
        textColor=COLORS['text_primary'],
        spaceAfter=30,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph(repo_name, repo_style))
    
    # Stats
    stats = [
        (f"{format_number(top_repo.get('capabilities', 0))}", "Capabilities"),
        (f"{format_number(top_repo.get('signals', 0))}", "Signals"),
        (f"{format_number(top_repo.get('filesScanned', 0))}", "Files"),
    ]
    
    for num, label in stats:
        stat_style = ParagraphStyle(
            'Stat',
            parent=styles['Normal'],
            fontSize=48,
            textColor=COLORS['accent_purple'],
            spaceAfter=5,
            alignment=TA_CENTER,
            fontName='Helvetica-Bold'
        )
        story.append(Paragraph(num, stat_style))
        
        label_style = ParagraphStyle(
            'StatLabel',
            parent=styles['Normal'],
            fontSize=16,
            textColor=COLORS['text_secondary'],
            spaceAfter=25,
            alignment=TA_CENTER,
            fontName='Helvetica'
        )
        story.append(Paragraph(label, label_style))
    
    # Fun fact
    fun_fact_style = ParagraphStyle(
        'FunFact',
        parent=styles['Normal'],
        fontSize=14,
        textColor=COLORS['accent_amber'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica'
    )
    story.append(Spacer(1, 0.2*inch))
    story.append(Paragraph("This repo alone could power dozens of products!", fun_fact_style))

def create_top_5_page(story, achievements):
    """Create page showing top 5 repos"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'WrappedTitle',
        parent=styles['Title'],
        fontSize=36,
        textColor=COLORS['highlight'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    story.append(Spacer(1, 0.3*inch))
    story.append(Paragraph("Your Top 5", title_style))
    story.append(Spacer(1, 0.1*inch))
    
    subtitle_style = ParagraphStyle(
        'Subtitle',
        parent=styles['Normal'],
        fontSize=18,
        textColor=COLORS['text_secondary'],
        spaceAfter=30,
        alignment=TA_CENTER,
        fontName='Helvetica'
    )
    story.append(Paragraph(f"These 5 repos contain {achievements['top_5_total']/achievements['total_capabilities']*100:.1f}% of all your capabilities", subtitle_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Top 5 list
    for idx, repo in enumerate(achievements['top_10_repos'][:5], 1):
        repo_name = repo.get('full_name', 'Unknown')
        if repo.get('private'):
            repo_name += ' 🔒'
        
        rank_style = ParagraphStyle(
            'Rank',
            parent=styles['Normal'],
            fontSize=24,
            textColor=COLORS['accent_cyan'],
            spaceAfter=5,
            alignment=TA_LEFT,
            fontName='Helvetica-Bold',
            leftIndent=20
        )
        story.append(Paragraph(f"#{idx}", rank_style))
        
        name_style = ParagraphStyle(
            'RepoName',
            parent=styles['Normal'],
            fontSize=20,
            textColor=COLORS['text_primary'],
            spaceAfter=5,
            alignment=TA_LEFT,
            fontName='Helvetica-Bold',
            leftIndent=60
        )
        story.append(Paragraph(repo_name, name_style))
        
        caps_style = ParagraphStyle(
            'Caps',
            parent=styles['Normal'],
            fontSize=16,
            textColor=COLORS['text_secondary'],
            spaceAfter=20,
            alignment=TA_LEFT,
            fontName='Helvetica',
            leftIndent=60
        )
        story.append(Paragraph(f"{format_number(repo.get('capabilities', 0))} capabilities", caps_style))

def create_product_potential_page(story, achievements):
    """Create page about product potential"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'WrappedTitle',
        parent=styles['Title'],
        fontSize=36,
        textColor=COLORS['highlight'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    story.append(Spacer(1, 0.3*inch))
    story.append(Paragraph("What You're Sitting On", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Big number
    big_num_style = ParagraphStyle(
        'BigNum',
        parent=styles['Normal'],
        fontSize=72,
        textColor=COLORS['accent_purple'],
        spaceAfter=10,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph(f"{format_number(achievements['product_potential'])}", big_num_style))
    
    label_style = ParagraphStyle(
        'Label',
        parent=styles['Normal'],
        fontSize=24,
        textColor=COLORS['text_primary'],
        spaceAfter=30,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("Potential Products", label_style))
    
    # Revenue estimates
    story.append(Spacer(1, 0.2*inch))
    
    revenue_style = ParagraphStyle(
        'Revenue',
        parent=styles['Normal'],
        fontSize=20,
        textColor=COLORS['accent_amber'],
        spaceAfter=15,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    conservative = achievements['product_potential'] * 12
    moderate = achievements['product_potential'] * 24
    aggressive = achievements['product_potential'] * 60
    
    story.append(Paragraph("Revenue Potential", revenue_style))
    story.append(Spacer(1, 0.1*inch))
    
    estimates = [
        (f"${format_number(conservative)}K", "Conservative (per year)"),
        (f"${format_number(moderate)}K", "Moderate (per year)"),
        (f"${format_number(aggressive)}K", "Aggressive (per year)"),
    ]
    
    for amount, label in estimates:
        amount_style = ParagraphStyle(
            'Amount',
            parent=styles['Normal'],
            fontSize=28,
            textColor=COLORS['highlight'],
            spaceAfter=5,
            alignment=TA_CENTER,
            fontName='Helvetica-Bold'
        )
        story.append(Paragraph(amount, amount_style))
        
        label_style_small = ParagraphStyle(
            'LabelSmall',
            parent=styles['Normal'],
            fontSize=14,
            textColor=COLORS['text_secondary'],
            spaceAfter=20,
            alignment=TA_CENTER,
            fontName='Helvetica'
        )
        story.append(Paragraph(label, label_style_small))

def create_developer_percentile_page(story, achievements):
    """Create page showing developer percentile"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'WrappedTitle',
        parent=styles['Title'],
        fontSize=36,
        textColor=COLORS['highlight'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    story.append(Spacer(1, 0.3*inch))
    story.append(Paragraph("You're in the", title_style))
    
    # Percentile
    percentile_style = ParagraphStyle(
        'Percentile',
        parent=styles['Normal'],
        fontSize=96,
        textColor=COLORS['accent_cyan'],
        spaceAfter=10,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("TOP 1%", percentile_style))
    
    label_style = ParagraphStyle(
        'Label',
        parent=styles['Normal'],
        fontSize=28,
        textColor=COLORS['text_primary'],
        spaceAfter=40,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("of Developers", label_style))
    
    # Fun facts
    story.append(Spacer(1, 0.3*inch))
    
    facts = [
        f"You've built {format_number(achievements['total_capabilities'])} capabilities",
        f"Across {format_number(achievements['total_repos'])} repositories",
        f"In {format_number(achievements['total_files'])} files",
        f"With {format_number(achievements['mega_repos'])} mega-repos (1000+ capabilities)",
    ]
    
    fact_style = ParagraphStyle(
        'Fact',
        parent=styles['Normal'],
        fontSize=16,
        textColor=COLORS['text_secondary'],
        spaceAfter=15,
        alignment=TA_CENTER,
        fontName='Helvetica'
    )
    
    for fact in facts:
        story.append(Paragraph(f"• {fact}", fact_style))

def create_year_summary_page(story, achievements):
    """Create final summary page"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'WrappedTitle',
        parent=styles['Title'],
        fontSize=42,
        textColor=COLORS['highlight'],
        spaceAfter=30,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    
    story.append(Spacer(1, 0.4*inch))
    story.append(Paragraph("2025", title_style))
    story.append(Spacer(1, 0.1*inch))
    
    subtitle_style = ParagraphStyle(
        'Subtitle',
        parent=styles['Normal'],
        fontSize=24,
        textColor=COLORS['text_primary'],
        spaceAfter=40,
        alignment=TA_CENTER,
        fontName='Helvetica'
    )
    story.append(Paragraph("What a year!", subtitle_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Summary points
    summary_points = [
        ("You built", f"{format_number(achievements['total_capabilities'])} capabilities"),
        ("Across", f"{format_number(achievements['total_repos'])} repositories"),
        ("In", f"{format_number(achievements['total_files'])} files"),
        ("With potential for", f"{format_number(achievements['product_potential'])} products"),
        ("And revenue potential of", f"${format_number(achievements['product_potential'] * 24)}K/year"),
    ]
    
    for label, value in summary_points:
        point_style = ParagraphStyle(
            'Point',
            parent=styles['Normal'],
            fontSize=18,
            textColor=COLORS['text_primary'],
            spaceAfter=20,
            alignment=TA_CENTER,
            fontName='Helvetica'
        )
        story.append(Paragraph(f"{label} <b>{value}</b>", point_style))
    
    story.append(Spacer(1, 0.3*inch))
    
    # Final message
    final_style = ParagraphStyle(
        'Final',
        parent=styles['Normal'],
        fontSize=20,
        textColor=COLORS['accent_amber'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("Your pantry is FULL.", final_style))
    story.append(Paragraph("Time to start cooking! 🍳", final_style))

def generate_wrapped_pdf():
    """Generate the Spotify Wrapped-style PDF"""
    # Load data
    data = load_catalog_data()
    achievements = calculate_achievements(data)
    
    # Create PDF
    script_dir = Path(__file__).parent
    output_path = script_dir.parent / 'docs' / 'repo-catalog' / 'PANTRY_WRAPPED_2025.pdf'
    
    doc = SimpleDocTemplate(
        str(output_path),
        pagesize=letter,
        rightMargin=0.75*inch,
        leftMargin=0.75*inch,
        topMargin=0.75*inch,
        bottomMargin=0.75*inch
    )
    
    story = []
    
    # Page 1: Title (handled by onFirstPage)
    story.append(PageBreak())
    
    # Page 2: Achievements
    create_achievement_page(story, achievements)
    story.append(PageBreak())
    
    # Page 3: Top Repo
    create_top_repo_page(story, achievements)
    story.append(PageBreak())
    
    # Page 4: Top 5
    create_top_5_page(story, achievements)
    story.append(PageBreak())
    
    # Page 5: Product Potential
    create_product_potential_page(story, achievements)
    story.append(PageBreak())
    
    # Page 6: Developer Percentile
    create_developer_percentile_page(story, achievements)
    story.append(PageBreak())
    
    # Page 7: Year Summary
    create_year_summary_page(story, achievements)
    
    # Build PDF with dark background on all pages
    doc.build(story, onFirstPage=create_wrapped_title_page, onLaterPages=set_dark_background)
    
    print(f"✅ Wrapped PDF generated: {output_path}")
    return output_path

if __name__ == '__main__':
    try:
        output_path = generate_wrapped_pdf()
        print(f"\n🎵 Your 2025 Developer Wrapped is ready!")
        print(f"   Location: {output_path}\n")
    except Exception as e:
        print(f"❌ Error generating PDF: {e}")
        import traceback
        traceback.print_exc()

