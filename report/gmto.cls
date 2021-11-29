\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{GMTO Notes}[2015/05/15 1.0]

\DeclareOption*{\PassOptionsToClass{\CurrentOption}{article}}
\ProcessOptions
\LoadClass[twoside,11pt]{article}

\RequirePackage{ifthen}
\RequirePackage[colorlinks=true,linkcolor=blue]{hyperref}
\RequirePackage{biblatex}
\RequirePackage{graphicx}
\RequirePackage{lastpage}
\RequirePackage{xspace}
\RequirePackage[table]{xcolor}
%\RequirePackage{colortbl}
\RequirePackage{tabu}
\RequirePackage{tikz}
% \usepackage{amsmath,amssymb}
% \usepackage{lastpage}
% \usepackage{booktabs}
% \usepackage{xspace}
% \usepackage{tikz}
% \usepackage{tabu}
% \usepackage{xcolor}
\definecolor{gmtblue}{HTML}{004181}

%\addtolength{\hoffset}{-0.5cm} 
\addtolength{\textwidth}{3cm}
\addtolength{\headheight}{5mm}
\addtolength{\evensidemargin}{-2cm}
\addtolength{\oddsidemargin}{-1cm}

\newcommand{\subtitle}[1]  {\renewcommand{\@subtitle}{#1}}
\newcommand{\DocID}[1]  {\renewcommand{\@id}{#1}}
\newcommand{\DocNumber}[1]  {\renewcommand{\@number}{#1}}
\newcommand{\DocVersion}[1] {\renewcommand{\@version}{#1}}
\newcommand{\DocStatus}[1] {\renewcommand{\@status}{#1}}
\newcommand{\@subtitle}{} 
\newcommand{\@status}{Draft} 
\newcommand{\@number}{} 
\newcommand{\@id}{GMT-XXX-\#\#\#\#} 
\newcommand{\@version}{1.0}
\@onlypreamble{\DocID}
\@onlypreamble{\DocNumber}
\@onlypreamble{\DocVersion}

\renewcommand\maketitle{
  \clearpage
  \thispagestyle{empty}
\begin{tikzpicture}[remember picture,overlay]
  \node[inner sep=0pt,anchor=center] at
       (current page.center)
      {\includegraphics[width=1\paperwidth,height=\paperheight]{GMTO_Title_Page_July_2016.jpg}};
\end{tikzpicture}
    \begin{tikzpicture}[remember picture,overlay]
      \node[yshift=54mm,xshift=2cm,text width=\linewidth,minimum height=29mm] at (current page.center)
      {{\Huge\textsf{\textbf{\textcolor{white}{\@title}}}}\\
      {\Large\textsf{\textbf{\textcolor{white}{\@subtitle}}}}};
    \end{tikzpicture}
    \begin{tikzpicture}[remember picture,overlay]
      \node[fill=white,inner sep=0,text width=0.75\linewidth,yshift=-1.5cm] at (current page.center)
      {{\small\tabulinesep=1em
        \begin{tabu} to \linewidth {|X[2,l]|X[1,l]|}\hline
     \textbf{Document ID}: \@id & \textbf{Version}: \@version  \\\hline
     \textbf{Date}: \@date              &  \textbf{Status}: \@status \\\hline
    \multicolumn{2}{|l|}{\textbf{Author(s)}:  \@author}
    \\\hline
    \end{tabu}}
 };
    \end{tikzpicture}
    \clearpage
    \setcounter{page}{1}
}

\newcommand{\oddinfo}{
  \begin{tabular}{rr}
    \multicolumn{2}{r}{\textbf{\@id}} \\
    {\small \@version} & {\small \@status} \\
    \multicolumn{2}{r}{{\small \today}}
  \end{tabular}
}

\newcommand{\eveninfo}{
  \begin{tabular}{ll}
    \multicolumn{2}{l}{\textbf{\@number}} \\
   \small \@status & \small \@version  \\
    \multicolumn{2}{l}{\small \today}
  \end{tabular}
}

\newcommand{\ps@dochead}{%
  \renewcommand{\@evenhead}{%
    \begin{tabu} to \linewidth {l@{}X[3,l]X[1,r]}
      \parbox[c]{1.5cm}{\includegraphics[width=\linewidth]{GMTO_Header_Logo_and_Separator.jpg}}
      &\textcolor{gmtblue}{\textsf{\textbf{ \@title}}}  & \oddinfo
    \end{tabu}
  }
  \renewcommand{\@oddhead}{%
    \begin{tabu} to \linewidth {l@{}X[3,l]X[1,r]}
      \parbox[c]{1.5cm}{\includegraphics[width=\linewidth]{GMTO_Header_Logo_and_Separator.jpg}}
      &\textcolor{gmtblue}{\textsf{\textbf{ \@title}}}  & \oddinfo
    \end{tabu}
  }

  \renewcommand{\@oddfoot}{
    \begin{minipage}{1.0\linewidth}
      \begin{center}
        {\color{gmtblue}\rule{\linewidth}{1pt}}
        Page \thepage\ of \pageref{LastPage}
      \end{center}
    \end{minipage}
  }
  \renewcommand{\@evenfoot}{
    \begin{minipage}{1.0\linewidth}
      \begin{center}
        {\color{gmtblue}\rule{\linewidth}{1pt}}
        Page \thepage\ of \pageref{LastPage}
      \end{center}
    \end{minipage}
  }
}
 \pagestyle{dochead}

 %\setlength{\headheight}{\baselineskip}
 %\setlength{\headsep}{3em}

%  \def\tableofcontents{
%    \section*{Contents}
%    \@starttoc{toc}
% %   \clearpage
%  }

 \newenvironment{revisions}
 {\rowcolors{1}{black!10}{black!20}
  \tabulinesep=5pt
  \begin{tabu} to \linewidth {p{1.4cm}p{3.1cm}p{1.75cm}p{2.1cm}p{3.5cm}p{1.75cm}}
  \textbf{Version} & \textbf{Date} & \textbf{Affected Sections} & \textbf{Change
    Request \#} & \textbf{Comments} & \textbf{Change Author} \\}
{\end{tabu}}

 \newenvironment{documents}
{\begin{tabular*}{\textwidth}{@{\extracolsep{\fill}}|l|p{0.1\textwidth}|p{0.3\textwidth}|l|l|}%
\hline
\textbf{Document ID} & \textbf{Source} & \textbf{Title} & \textbf{Version} & \textbf{Date} \\\hline}
{\hline
\end{tabular*}}
