#include "viewportwidget.h"
#include <QHBoxLayout>

struct ViewPortWidget::ViewPortWidgetPrivate {
    std::unique_ptr<QOpenGLWidget> opengl_widget;
    std::unique_ptr<QHBoxLayout> main_layout;

    ViewPortWidgetPrivate() {
        this->opengl_widget = std::make_unique<QOpenGLWidget>();
        this->main_layout = std::make_unique<QHBoxLayout>();
    }
};

ViewPortWidget::ViewPortWidget(QWidget *parent)
    : QWidget{parent},
      _p{new ViewPortWidgetPrivate} {

    this->setMinimumSize(800, 600);

    auto main_layout = _p->main_layout.get();
    auto opengl_widget = _p->opengl_widget.get();

    this->setLayout(main_layout);
    main_layout->addWidget(opengl_widget, 1);

    QSurfaceFormat format;
    format.setDepthBufferSize(24);
    format.setStencilBufferSize(8);
    format.setVersion(1, 1);
    format.setProfile(QSurfaceFormat::CoreProfile);
    opengl_widget->setFormat(format);
}

ViewPortWidget::~ViewPortWidget() {
    if (_p) delete _p;
}
