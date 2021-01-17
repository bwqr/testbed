import {AfterViewInit, Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {Experiment} from '../../models';
import {ActivatedRoute} from '@angular/router';
import {switchMap} from 'rxjs/operators';
import CodeMirror from 'codemirror';
import * as python from 'codemirror/mode/python/python.js';

// This expression is required since we want python to be imported
// tslint:disable-next-line:no-unused-expression
python;

@Component({
  selector: 'app-experiment',
  templateUrl: './experiment.component.html',
  styleUrls: ['./experiment.component.scss']
})
export class ExperimentComponent extends MainComponent implements OnInit, AfterViewInit {

  experiment: Experiment;

  @ViewChild('code') code: ElementRef;

  codeMirror: any;

  get isPageReady(): boolean {
    return !!this.experiment;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap(params => this.viewModel.experiment(params.id))
      ).subscribe(experiment => this.experiment = experiment)
    );
  }

  ngAfterViewInit(): void {
    this.codeMirror = CodeMirror(this.code.nativeElement, {
      value: 'print("Hello World")',
      mode: 'python',
      lineNumbers: true,
      lineWiseCopyCut: true,
    });
  }
}
