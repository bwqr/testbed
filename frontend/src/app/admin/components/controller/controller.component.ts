import {Component, OnDestroy, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../../experiment/services/experiment-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {catchError, switchMap} from 'rxjs/operators';
import {SlimController} from '../../../experiment/models';
import {AdminViewModelService} from '../../services/admin-view-model.service';
import {interval, Subscription} from 'rxjs';
import {formats} from '../../../../defs';
import {line} from 'd3-shape/src/index.js';

@Component({
  selector: 'app-receiver-values',
  templateUrl: './controller.component.html',
  styleUrls: ['./controller.component.scss']
})
export class ControllerComponent extends MainComponent implements OnInit, OnDestroy {
  private static MAX_BUFFER_LENGTH = 20;

  controller: SlimController;

  receiverValuesSub: Subscription;

  receiverValues: number[][] = [];

  formats = formats;

  get isPageReady(): boolean {
    return !!this.controller;
  }


  constructor(
    private viewModel: AdminViewModelService,
    private experimentViewModel: ExperimentViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap((params) => this.experimentViewModel.controller(params.id)),
      )
        .subscribe(controller => {
          this.controller = controller;

          if (this.receiverValuesSub) {
            this.receiverValuesSub.unsubscribe();
          }

          this.receiverValuesSub = interval(2000).pipe(
            switchMap(_ => this.viewModel.controllerReceiversValues(controller.id)),
            catchError((_, caught) => caught)
          )
            .subscribe(res => {
              if (!res.values) {
                return;
              }

              if (res.values.length !== this.receiverValues.length) {
                this.receiverValues = res.values.map(v => [v]);
                this.buildGraphs(this.receiverValues);
              } else {
                this.updateGraphs(res.values);
              }
            });
        })
    );
  }

  ngOnDestroy(): void {
    super.ngOnDestroy();

    if (this.receiverValuesSub) {
      this.receiverValuesSub.unsubscribe();
    }
  }

  buildGraphs(values: number[][]): void {
    const container = document.getElementById('svg-container');

    while (container.firstChild) {
      container.removeChild(container.lastChild);
    }

    // create receiver svgs
    values.forEach(_ => {
      const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svg.classList.add('line');
      container.appendChild(svg);
    });

    container.childNodes.forEach((node, i) => {
      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      node.appendChild(path);

      path.setAttribute('d', line()(values[i].map((v, idx) => [0, v])));
    });
  }

  updateGraphs(values: number[]): void {
    values.forEach((v, i) => {
      if (this.receiverValues[i].length > ControllerComponent.MAX_BUFFER_LENGTH) {
        this.receiverValues[i].shift();
      }

      this.receiverValues[i].push(v);
    });

    const container = document.getElementById('svg-container');

    container.childNodes.forEach((c, i) => {
      (c.firstChild as any).setAttribute('d', line()(this.receiverValues[i].map((v, idx) => [idx * 15, v])));
    });
  }
}
